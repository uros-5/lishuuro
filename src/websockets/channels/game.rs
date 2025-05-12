use std::{hash::Hash, sync::Arc};

use bson::DateTime;
use chrono::{DateTime as DateTime2, Utc};
use chrono::{FixedOffset, TimeDelta};
use serde::Serialize;
use shuuro::position::Outcome;
use shuuro::PieceType;
use shuuro::{
    attacks::Attacks,
    bitboard::BitBoard,
    position::{Board, Placement, Play, Rules, Sfen},
    Color, Move, Selection, Square,
};
use tokio::sync::mpsc::{self, Sender};

use crate::{
    database::{
        clock::queries::{add_game_to_db, remove_game, update_entire_game},
        model::ShuuroGame,
        Database,
    },
    websockets::handler::WsMessage,
};

use super::game_reguests::GameRequestMessage;
use super::tv::TvMessage;
use super::{
    clock::{clock_task, ClockMessage},
    game_reguests::{GameRequest, TypeOfGame},
    games::GamesMessage,
    message_types::MessageType,
    players::PlayersMessage,
    watchers::{SendTo, Watchers},
    WsState,
};

pub async fn game_task<S, B, A, P>(
    db: Arc<Database>,
    ws: Arc<WsState>,
    game_request: GameRequest,
    id: String,
    caller: String,
) where
    S: Square + Hash + Send + 'static,
    B: BitBoard<S>,
    A: Attacks<S, B>,
    P: Sized
        + Clone
        + Board<S, B, A>
        + Sfen<S, B, A>
        + Placement<S, B, A>
        + Play<S, B, A>
        + Rules<S, B, A>
        + Send
        + 'static,
{
    // A::init();
    let mut other_player = {
        match game_request.game_type {
            TypeOfGame::VsFriend(ref player) => player.to_string().replace(' ', ""),
            TypeOfGame::VsAi(_) => "ai".to_string(),
        }
    };
    if other_player == "ai" {
        return;
    }
    let colors = game_request.colors(&caller, &other_player);
    let mut watchers = Watchers::new();
    let game = ShuuroGame::from((&game_request, &colors, id.as_str()));
    let mut game = add_game_to_db(&db.mongo.games, game).await;
    let (mut selection, mut placement, mut fight) =
        (Selection::<S>::default(), P::new(), P::new());

    selection.update_variant(game.variant);
    placement.update_variant(game.variant);
    fight.update_variant(game.variant);

    let mut abort_game_counter = 0;

    if let Some(subvariant) = game.sub_variant {
        let stage = subvariant.starting_stage();
        let sfen = subvariant.starting_position();
        game.current_stage = stage;
        if stage == 2 {
            fight.set_sfen(sfen).expect("fen build failed");
            fight.generate_plinths();
            let sfen = fight.generate_sfen();
            // game.history.2.push(String::from(&sfen));
            game.game_start = String::from(&sfen);
            game.sfen = sfen;
            game.tc.update_stage(2);
        } else if stage == 1 {
            placement.set_sfen(sfen).expect("fen build failed");
            placement.generate_plinths();
            game.sfen = placement.generate_sfen();
            game.placement_start = String::from(sfen);
            // game.history.1.push(String::from(&game.sfen));
            game.tc.update_stage(1);
        }
    }
    let (send, mut recv) = mpsc::channel(30);
    let _ = ws
        .games
        .send(GamesMessage::InsertGame {
            channel: send.clone(),
            id: id.to_string(),
        })
        .await;
    let _ = ws
        .players
        .send(PlayersMessage::Redirect {
            game: id.to_string(),
            player: caller.to_string(),
        })
        .await;
    let mut started = false;
    let clock_task = clock_task(send.clone()).await;
    let mut current_interval = 15_000;
    tokio::spawn(async move {
        while let Some(message) = recv.recv().await {
            match message {
                GameMessage::Join(player, sender) => {
                    if watchers.players.len() == 10 {
                        continue;
                    }
                    watchers.add_watcher(player.clone(), sender);
                    if started == false && &player != &caller {
                        if other_player == "" || &other_player == &player {
                            other_player = player.clone();
                            let index = game
                                .players
                                .iter()
                                .position(|item| item == &caller)
                                .unwrap();
                            let index = {
                                if index == 0 {
                                    1
                                } else {
                                    0
                                }
                            };
                            game.players[index] = other_player.to_string();

                            game.tc.update_stage(game.current_stage);
                            started = true;
                            let message = WsMessage::Message(
                                serde_json::json!(StartClock {
                                    t: MessageType::StartClock,
                                    players: game.players.clone(),
                                    click: Utc::now().into()
                                })
                                .to_string(),
                            );
                            watchers.notify(message, SendTo::Everyone).await;
                            let _ = ws
                                .game_requests
                                .send(GameRequestMessage::AddActivePlayer(
                                    other_player.to_string(),
                                ))
                                .await;
                        }
                    }
                }
                GameMessage::Leave(player) => {
                    watchers.remove_watcher(&player);
                }
                GameMessage::GetGame(sender) => {
                    let mut game = game.clone();
                    game.hands = ["".into(), "".into()];
                    let _ = sender.send(game.clone());
                }
                GameMessage::GetHand(player) => {
                    if !started {
                        continue;
                    }

                    if game.current_stage != 0 {
                        continue;
                    }
                    let Some(index) = player_index(&game.players, &player) else {
                        continue;
                    };
                    let selection = game.hands[index].to_owned();
                    let hand = PlayerSelection {
                        t: MessageType::GetHand,
                        hand: selection,
                    };
                    let hand = serde_json::json!(hand).to_string();
                    watchers
                        .notify(
                            WsMessage::Message(hand),
                            SendTo::Players {
                                list: vec![player],
                                to_others: false,
                            },
                        )
                        .await;
                }
                GameMessage::GameMove {
                    ref player,
                    game_move,
                } => {
                    if !started {
                        continue;
                    }
                    let Some(index) = player_index(&game.players, &player) else {
                        continue;
                    };

                    let Some(m) = Move::<S>::from_sfen(&game_move) else {
                        if game.current_stage != 0 {
                            continue;
                        }
                        let me = Color::from(index);
                        selection.confirm(me);

                        confirm_selection(
                            me,
                            &watchers,
                            &mut game,
                            &mut selection,
                            &mut placement,
                            ws.clone(),
                        )
                        .await;
                        continue;
                    };
                    if let Move::Select { piece } = m {
                        if game.current_stage != 0 {
                            continue;
                        }
                        let me = Color::from(index);
                        if piece.color != me {
                            continue;
                        }
                        let Some(_) = selection.play(m) else {
                            continue;
                        };

                        game.draws = [false, false];
                        game.hands[index] = selection.to_sfen(me, false);

                        confirm_selection(
                            me,
                            &watchers,
                            &mut game,
                            &mut selection,
                            &mut placement,
                            ws.clone(),
                        )
                        .await;
                    } else if let Move::Put { to, piece } = m {
                        if game.current_stage != 1 {
                            continue;
                        }
                        let color = Color::from(index);
                        if placement.side_to_move() != color {
                            continue;
                        }

                        let Some(clocks) = game.tc.play(index) else {
                            continue;
                        };
                        game.clocks = game.tc.clocks;
                        game.last_clock = DateTime::now();

                        if color != piece.color {
                            continue;
                        }
                        let Some(sfen) = placement.place(piece, to) else {
                            continue;
                        };
                        game.draws = [false, false];
                        let mut first_move_error = false;
                        let next_stage = {
                            let mut completed: [bool; 3] = [false, false, false];
                            let color_iter = Color::iter();
                            for i in color_iter {
                                completed[i.index()] =
                                    placement.is_hand_empty(i, PieceType::Plinth);
                            }
                            completed[2] = true;
                            !completed.contains(&false)
                        };
                        if next_stage {
                            first_move_error = {
                                game.current_stage = 2;
                                game.tc.update_stage(2);
                                game.last_clock = DateTime::now();
                                let sfen = placement.generate_sfen();
                                let outcome = fight.set_sfen(&sfen);
                                if let Ok(outcome) = outcome {
                                    update_status(&mut game, &outcome);
                                }
                                game.game_start = sfen.to_string();
                                // game.history.2.push(sfen.to_string());
                                fight.in_check(fight.side_to_move().flip())
                            };
                        }
                        game.side_to_move = placement.side_to_move() as u8;
                        game.sfen = sfen.to_string();
                        game.hands = [
                            placement.get_hand(Color::White, false),
                            placement.get_hand(Color::Black, false),
                        ];
                        game.history.1.push(m.to_fen());
                        let message = PlacePiece {
                            clocks,
                            first_move_error,
                            next_stage,
                            sfen: m.to_fen(),
                            t: MessageType::PlacePiece,
                        };
                        let message = WsMessage::Message(
                            serde_json::json!(message).to_string(),
                        );
                        watchers.notify(message, SendTo::Everyone).await;
                        let _ = ws
                            .tv
                            .send(TvMessage::Move {
                                id: game._id.to_string(),
                                sfen: game.sfen.to_string(),
                                first_move_error,
                            })
                            .await;
                        if first_move_error {
                            update_entire_game(&db.mongo.games, &game).await;
                            close_game(
                                clock_task,
                                fight.side_to_move().flip().index() as u8,
                                game.status,
                                &watchers,
                                ws.game_requests.clone(),
                                &game,
                            )
                            .await;
                            let _ = ws
                                .tv
                                .send(TvMessage::Remove {
                                    id: game._id.to_string(),
                                })
                                .await;
                            break;
                        }
                    } else if let Move::Normal {
                        from,
                        to: _,
                        placed: _,
                    } = m
                    {
                        if game.current_stage != 2 {
                            continue;
                        }
                        let color = Color::from(index);
                        if fight.side_to_move() != color {
                            continue;
                        }

                        let Some(clocks) = game.tc.play(index) else {
                            continue;
                        };
                        game.clocks = game.tc.clocks;
                        game.last_clock = DateTime::now();
                        let Some(piece) = fight.piece_at(from) else {
                            continue;
                        };
                        if color != piece.color {
                            continue;
                        }

                        let Ok(outcome) = fight.play(&game_move) else {
                            continue;
                        };
                        update_status(&mut game, outcome);

                        let side_to_move = fight.side_to_move().flip() as u8;
                        game.side_to_move = side_to_move;
                        game.sfen = fight.get_sfen_history().first();
                        game.history.2.push(game_move.to_string());
                        let message = MovePiece {
                            clocks,
                            status: game.status,
                            result: 2,
                            game_move,
                            t: MessageType::MovePiece,
                        };
                        let message = WsMessage::Message(
                            serde_json::json!(message).to_string(),
                        );
                        watchers.notify(message, SendTo::Everyone).await;
                        let _ = ws
                            .tv
                            .send(TvMessage::Move {
                                id: game._id.to_string(),
                                sfen: game.sfen.to_string(),
                                first_move_error: false,
                            })
                            .await;
                        if game.status > 0 {
                            update_entire_game(&db.mongo.games, &game).await;
                            close_game(
                                clock_task,
                                game.side_to_move as u8,
                                game.status,
                                &watchers,
                                ws.game_requests.clone(),
                                &game,
                            )
                            .await;
                            let _ = ws
                                .tv
                                .send(TvMessage::Remove {
                                    id: game._id.to_string(),
                                })
                                .await;
                            break;
                        }
                    }
                }
                GameMessage::Draw(player) => {
                    if !started {
                        continue;
                    }

                    let Some(index) = player_index(&game.players, &player) else {
                        continue;
                    };
                    game.draws[index] = true;
                    if game.draws == [true, true] {
                        game.status = 5;
                        update_entire_game(&db.mongo.games, &game).await;
                        close_game(
                            clock_task,
                            2,
                            5,
                            &watchers,
                            ws.game_requests.clone(),
                            &game,
                        )
                        .await;
                        let _ = ws
                            .tv
                            .send(TvMessage::Remove {
                                id: game._id.to_string(),
                            })
                            .await;
                        break;
                    }
                    let draw = GameDraw {
                        t: MessageType::Draw,
                        draw_offer: true,
                        end: -2,
                    };
                    let draw = serde_json::json!(draw).to_string();
                    watchers
                        .notify(
                            WsMessage::Message(draw),
                            SendTo::Players {
                                list: vec![game.players
                                    [Color::from(index).flip() as usize]
                                    .to_string()],
                                to_others: false,
                            },
                        )
                        .await;
                }
                GameMessage::Resign(player) => {
                    let Some(index) = player_index(&game.players, &player) else {
                        continue;
                    };
                    game.status = 7;
                    game.result = index as u8;
                    game.tc.play(index);
                    game.last_clock = DateTime::now();
                    update_entire_game(&db.mongo.games, &game).await;
                    close_game(
                        clock_task,
                        game.result,
                        7,
                        &watchers,
                        ws.game_requests.clone(),
                        &game,
                    )
                    .await;

                    let _ = ws
                        .tv
                        .send(TvMessage::Remove {
                            id: game._id.to_string(),
                        })
                        .await;
                    break;
                }
                GameMessage::Abort => {
                    let _id =
                        remove_game(&db.mongo.games, game._id.to_string()).await;
                    close_game(
                        clock_task,
                        2,
                        10,
                        &watchers,
                        ws.game_requests.clone(),
                        &game,
                    )
                    .await;

                    let _ = ws
                        .tv
                        .send(TvMessage::Remove {
                            id: game._id.to_string(),
                        })
                        .await;
                    break;
                }
                GameMessage::CheckClock => {
                    let mut stm = game.side_to_move;
                    if !started {
                        abort_game_counter += 1;
                        if abort_game_counter == 4 {
                            let _ = send.send(GameMessage::Abort).await;
                        }
                        continue;
                    }
                    if game.current_stage == 0 {
                        let confirmed = [
                            selection.is_confirmed(Color::White),
                            selection.is_confirmed(Color::Black),
                        ];
                        if !confirmed.contains(&true) {
                            let clock = game.tc.current_duration(1);
                            if clock.is_none() {
                                game.result = 2;
                                game.status = 8;
                                game.tc.set_to_zero(Color::White);
                                game.tc.set_to_zero(Color::Black);
                                update_entire_game(&db.mongo.games, &game).await;
                                close_game(
                                    clock_task,
                                    2,
                                    8,
                                    &watchers,
                                    ws.game_requests.clone(),
                                    &game,
                                )
                                .await;

                                let _ = ws
                                    .tv
                                    .send(TvMessage::Remove {
                                        id: game._id.to_string(),
                                    })
                                    .await;

                                break;
                            }
                        }
                        let not_confirmed =
                            confirmed.iter().position(|item| item == &false);
                        match not_confirmed {
                            Some(i) => {
                                stm = i as u8;
                            }
                            None => {
                                stm = game.side_to_move;
                            }
                        };
                    }
                    let Some(clock) = game.tc.current_duration(stm.into()) else {
                        game.result = stm as u8;
                        game.status = 8;
                        game.tc.set_to_zero(Color::from(stm as usize));
                        update_entire_game(&db.mongo.games, &game).await;
                        close_game(
                            clock_task,
                            stm as u8,
                            8,
                            &watchers,
                            ws.game_requests.clone(),
                            &game,
                        )
                        .await;

                        let _ = ws
                            .tv
                            .send(TvMessage::Remove {
                                id: game._id.to_string(),
                            })
                            .await;

                        break;
                    };
                    let mut players = game.tc.clocks.clone();
                    players[stm as usize] = clock;

                    let interval = clocks(players);
                    if current_interval == interval {
                        continue;
                    }
                    current_interval = interval;
                    let _ = clock_task
                        .send(ClockMessage::IncreaseInterval(current_interval))
                        .await;

                    continue;
                }
            }
        }
    });
}

fn update_status(game: &mut ShuuroGame, outcome: &Outcome) {
    match outcome {
        Outcome::Check { color: _ } => {
            game.status = -1;
        }
        Outcome::Stalemate => {
            game.status = 3;
            game.result = 2;
        }
        Outcome::DrawByAgreement => {
            game.status = 5;
            game.result = 2;
        }
        Outcome::DrawByRepetition => {
            game.status = 4;
            game.result = 2;
        }
        Outcome::DrawByMaterial => {
            game.status = 6;
            game.result = 2;
        }
        Outcome::Checkmate { color } => {
            game.status = 1;
            game.result = *color as u8;
        }
        Outcome::MoveOk => {
            game.status = -1;
        }
        Outcome::MoveNotOk => {
            game.status = -2;
        }
        Outcome::Resign { color } => {
            game.status = 7;
            game.result = *color as u8;
        }
        Outcome::LostOnTime { color } => {
            game.status = 8;
            game.result = *color as u8;
        }
        Outcome::FirstMoveError { color } => {
            game.status = 9;
            game.result = *color as u8;
        }
    }
}

pub enum GameMessage {
    Join(String, Sender<WsMessage>),
    Leave(String),
    GetGame(tokio::sync::oneshot::Sender<ShuuroGame>),
    GetHand(String),
    GameMove { player: String, game_move: String },
    Draw(String),
    Resign(String),
    Abort,
    CheckClock,
}

pub enum MoveType {
    Selection,
    Placement,
    MovePiece,
}

#[derive(Serialize)]
pub struct GameEnd {
    t: MessageType,
    result: u8,
    status: i32,
}

#[derive(Serialize)]
pub struct PlayerSelection {
    t: MessageType,
    hand: String,
}

#[derive(Serialize)]
pub struct GameDraw {
    t: MessageType,
    draw_offer: bool,
    end: i8,
}

#[derive(Serialize)]
pub struct ConfirmSelection {
    t: MessageType,
    confirmed: [bool; 2],
}

#[derive(Serialize, Clone)]
pub struct RedirectToPlacement {
    t: MessageType,
    pub id: String,
    last_clock: DateTime2<FixedOffset>,
    players: [String; 2],
    pub sfen: String,
    variant: u8,
}

#[derive(Serialize)]
pub struct PlacePiece {
    t: MessageType,
    clocks: [u64; 2],
    first_move_error: bool,
    next_stage: bool,
    sfen: String,
}

#[derive(Serialize)]
pub struct MovePiece {
    t: MessageType,
    clocks: [u64; 2],
    status: i32,
    result: u8,
    game_move: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct StartClock {
    t: MessageType,
    players: [String; 2],
    click: DateTime2<FixedOffset>,
}

fn clocks(clocks: [TimeDelta; 2]) -> u64 {
    let clock;
    if clocks[1] < clocks[0] {
        clock = clocks[1];
    } else {
        clock = clocks[0];
    }
    if clock.num_seconds() < 10 {
        return 500;
    } else if clock.num_minutes() < 1 {
        return 2000;
    } else if clock.num_minutes() < 5 {
        return 5000;
    }
    10_000
}

async fn close_game(
    clock_task: mpsc::Sender<ClockMessage>,
    result: u8,
    status: i32,
    watchers: &Watchers,
    requests: mpsc::Sender<GameRequestMessage>,
    game: &ShuuroGame,
) {
    let _ = clock_task.send(ClockMessage::StopClock).await;

    let message = GameEnd {
        t: MessageType::GameEnd,
        result,
        status,
    };
    watchers
        .notify(
            WsMessage::Message(serde_json::json!(message).to_string()),
            SendTo::Everyone,
        )
        .await;
    let _ = requests
        .send(GameRequestMessage::RemovePlayers(game.players.clone()))
        .await;
}

pub fn player_index(p: &[String; 2], u: &String) -> Option<usize> {
    p.iter().position(|x| x == u)
}

pub async fn confirm_selection<S, B, A, P>(
    me: Color,
    watchers: &Watchers,
    game: &mut ShuuroGame,
    selection: &mut Selection<S>,
    placement: &mut P,
    ws: Arc<WsState>,
) where
    S: Square + Hash + Send + 'static,
    B: BitBoard<S>,
    A: Attacks<S, B>,
    P: Sized
        + Clone
        + Board<S, B, A>
        + Sfen<S, B, A>
        + Placement<S, B, A>
        + Play<S, B, A>
        + Rules<S, B, A>
        + Send
        + 'static,
{
    if selection.is_confirmed(me) {
        let msg = ConfirmSelection {
            t: MessageType::ConfirmSelection,
            confirmed: [
                selection.is_confirmed(Color::White),
                selection.is_confirmed(Color::Black),
            ],
        };
        let msg = serde_json::json!(msg).to_string();
        watchers
            .notify(WsMessage::Message(msg), SendTo::Everyone)
            .await;
        game.clocks = game.tc.select(me);
        if !selection.is_confirmed(me.flip()) {
            return;
        }
        game.current_stage = 1;
        game.tc.update_stage(1);
        game.last_clock = DateTime::now();
        {
            let w = selection.to_sfen(Color::White, false);
            let b = selection.to_sfen(Color::Black, false);
            let hand = format!("{}{}", &w, &b);
            game.hands = [w, b];
            let sfen = P::empty_placement_board();
            placement.set_sfen(&format!("{} {} 1", sfen, hand)).ok();
        };
        placement.generate_plinths();
        game.sfen = placement.generate_sfen();
        game.side_to_move = 0;
        game.placement_start = game.sfen.to_string();
        // game.history.1.push(game.sfen.to_string());
        let redirect = RedirectToPlacement {
            t: MessageType::RedirectToGame,
            id: game._id.to_string(),
            last_clock: Utc::now().into(),
            players: game.players.clone(),
            sfen: game.sfen.to_string(),
            variant: game.variant as u8,
        };
        watchers
            .notify(
                WsMessage::Message(serde_json::json!(redirect.clone()).to_string()),
                SendTo::Everyone,
            )
            .await;
        let _ = ws.tv.send(TvMessage::Add(redirect.into())).await;
    }
}
