use rand::{rng, seq::IndexedRandom};
use shuuro::{
    attacks::Attacks,
    bitboard::BitBoard,
    position::{Board, Outcome, Placement, Play, Rules, Sfen},
    Color, Move, Piece, PieceType, Selection, Square, Variant,
};
use shuuro_engine::{Engine, EngineDefs};
use std::{env, f32::INFINITY, hash::Hash, marker::PhantomData, sync::Arc};
use tokio::{
    sync::mpsc::{self, Sender},
    time,
};

use crate::websockets::{channels::game::MovePiece, handler::WsMessage};

use super::game::{GameDraw, GameEnd, GameMessage, PlacePiece, RedirectToPlacement};

pub enum AiChannelMessage<S, B, A, P>
where
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
    SetSelection(Selection<S>),
    SetPlacement(P),
    SetFight(P),
    End,
    Null {
        _ph: PhantomData<S>,
        _ph2: PhantomData<B>,
        _ph3: PhantomData<A>,
    },
}

pub async fn ai_channel<
    S,
    B,
    A,
    P,
    E,
    D,
    const BITBOARD_SIZE: usize,
    const LEN: usize,
    const RANK: usize,
>(
    game_channel: mpsc::Sender<GameMessage>,
    mut depth: i32,
    player: Color,
    position: P,
    selection: Selection<S>,
    current_stage: u8,
    pockets: Arc<Pockets>,
) where
    S: Square + Hash + Send + 'static + std::marker::Sync,
    B: BitBoard<S> + std::marker::Send + 'static + std::marker::Sync,
    A: Attacks<S, B> + std::marker::Send + 'static + std::marker::Sync,
    P: Sized
        + Clone
        + Board<S, B, A>
        + Sfen<S, B, A>
        + Placement<S, B, A>
        + Play<S, B, A>
        + Rules<S, B, A>
        + Send
        + 'static
        + std::fmt::Display
        + std::marker::Sync,
    D: EngineDefs<S, B, LEN> + std::marker::Send + std::marker::Sync + 'static,
    E: Engine<S, B, A, P, D, LEN, BITBOARD_SIZE, RANK>
        + std::marker::Send
        + 'static
        + std::marker::Sync,
{
    if depth < 0 || depth > 3 {
        depth = 1;
    } else if depth == 3
        && (selection.variant() == Variant::Shuuro
            || selection.variant() == Variant::ShuuroFairy)
    {
        depth = 2;
    }
    let (player_sender, mut player_recv) = mpsc::channel(20);
    let mut ai = AiChannel::<S, B, A, P, E, D, BITBOARD_SIZE, LEN, RANK>::new(
        player,
        position,
        selection,
        game_channel,
        depth,
        pockets,
    );
    tokio::spawn(async move {
        ai.join(player_sender).await;
        if current_stage == 0 {
            ai.select().await;
        } else if current_stage == 1 {
            ai.place_piece(None).await;
        } else if current_stage == 2 {
            ai.last_move = " ".to_string();
            ai.move_piece("").await;
        }
        while let Some(WsMessage::Message(message)) = player_recv.recv().await {
            if let Ok(_message) = serde_json::from_str::<GameDraw>(&message) {
                ai.draw().await;
                break;
            } else if let Ok(mv) = serde_json::from_str::<MovePiece>(&message) {
                ai.move_piece(&mv.game_move).await;
            } else if let Ok(message) =
                serde_json::from_str::<RedirectToPlacement>(&message)
            {
                ai.redirect_to_placement(&message).await;
            } else if let Ok(state) = serde_json::from_str::<GameEnd>(&message) {
                if state.status > 0 {
                    break;
                }
            } else if let Ok(mv) = serde_json::from_str::<PlacePiece>(&message) {
                if let Some(Move::Put { to, piece }) =
                    Move::<S>::from_sfen(mv.sfen.as_ref())
                {
                    ai.place_piece(Some((piece, to))).await;
                    ai.next_stage().await;
                }
            }
        }
    });
}

fn uppercase(hand: &String, player: Color) -> String {
    if player == Color::White {
        hand.to_uppercase()
    } else {
        hand.to_lowercase()
    }
}

pub struct AiChannel<
    S,
    B,
    A,
    P,
    E,
    D,
    const BITBOARD_SIZE: usize,
    const LEN: usize,
    const RANK: usize,
> where
    S: Square + Hash + Send + 'static,
    B: BitBoard<S> + std::marker::Send + 'static + std::marker::Sync,
    A: Attacks<S, B> + std::marker::Send + 'static,
    P: Sized
        + Clone
        + Board<S, B, A>
        + Sfen<S, B, A>
        + Placement<S, B, A>
        + Play<S, B, A>
        + Rules<S, B, A>
        + Send
        + 'static
        + std::fmt::Display
        + std::marker::Sync,
    D: EngineDefs<S, B, LEN>,
    E: Engine<S, B, A, P, D, LEN, BITBOARD_SIZE, RANK> + std::marker::Send + 'static,
{
    pub position: P,
    pub selection: Selection<S>,
    pub engine: E,
    pub game_channel: Sender<GameMessage>,
    pub player: Color,
    pub last_move: String,
    depth: i32,
    placement_finished: bool,
    pockets: Arc<Pockets>,
    _ph1: PhantomData<S>,
    _ph2: PhantomData<B>,
    _ph3: PhantomData<A>,
    _ph4: PhantomData<D>,
}

impl<
        S,
        B,
        A,
        E,
        P,
        D,
        const BITBOARD_SIZE: usize,
        const LEN: usize,
        const RANK: usize,
    > AiChannel<S, B, A, P, E, D, BITBOARD_SIZE, LEN, RANK>
where
    S: Square + Hash + Send + 'static,
    B: BitBoard<S> + std::marker::Send + 'static + std::marker::Sync,
    A: Attacks<S, B> + std::marker::Send + 'static,
    P: Sized
        + Clone
        + Board<S, B, A>
        + Sfen<S, B, A>
        + Placement<S, B, A>
        + Play<S, B, A>
        + Rules<S, B, A>
        + Send
        + 'static
        + std::fmt::Display
        + std::marker::Sync,
    D: EngineDefs<S, B, LEN>,
    E: Engine<S, B, A, P, D, LEN, BITBOARD_SIZE, RANK> + std::marker::Send + 'static,
{
    pub fn new(
        player: Color,
        position: P,
        selection: Selection<S>,
        game_channel: mpsc::Sender<GameMessage>,
        depth: i32,
        pockets: Arc<Pockets>,
    ) -> Self {
        let engine = E::new();
        Self {
            selection,
            position,
            engine,
            player,
            _ph1: PhantomData,
            _ph2: PhantomData,
            _ph3: PhantomData,
            _ph4: PhantomData,
            game_channel,
            depth,
            placement_finished: false,
            last_move: String::from("____"),
            pockets,
        }
    }

    fn get_pocket(&self, variant: Variant, player: Color) -> String {
        let mut rng = rng();
        let hand = match variant {
            Variant::Shuuro => self
                .pockets
                .large
                .choose(&mut rng)
                .unwrap_or(&self.pockets.large[0]),
            Variant::ShuuroFairy => self
                .pockets
                .large_fairy
                .choose(&mut rng)
                .unwrap_or(&self.pockets.large_fairy[0]),
            Variant::Standard => self
                .pockets
                .standard
                .choose(&mut rng)
                .unwrap_or(&self.pockets.standard[0]),
            Variant::StandardFairy => self
                .pockets
                .standard_fairy
                .choose(&mut rng)
                .unwrap_or(&self.pockets.standard_fairy[0]),
            Variant::ShuuroMini => self
                .pockets
                .mini
                .choose(&mut rng)
                .unwrap_or(&self.pockets.mini[0]),
            Variant::ShuuroMiniFairy => self
                .pockets
                .mini_fairy
                .choose(&mut rng)
                .unwrap_or(&self.pockets.mini_fairy[0]),
        };
        uppercase(hand, player)
    }

    pub fn update_variant(&mut self, variant: Variant) {
        self.position.update_variant(variant);
    }

    async fn draw(&self) {
        let _ = self
            .game_channel
            .send(GameMessage::Draw("AI".to_string()))
            .await;
    }

    async fn move_piece(&mut self, move_: &str) {
        let mut mv = Ok(&Outcome::MoveOk);
        if move_ == self.last_move {
            return;
        }
        if move_ != "" {
            mv = self.position.play(move_);
        } else if self.position.side_to_move() != self.player {
            return;
        }
        match mv {
            Ok(_) => {
                let mv = self.engine.alpha_beta_search(
                    &self.position,
                    self.depth,
                    -INFINITY as i32,
                    INFINITY as i32,
                    self.position.side_to_move() == Color::White,
                    true,
                );
                if let Some(mv) = mv.best_move(&self.position) {
                    let outcome = self.position.play(&mv.to_fen());
                    if let Ok(_) = outcome {
                        let _ = self
                            .game_channel
                            .send(GameMessage::GameMove {
                                player: String::from("AI"),
                                game_move: mv.to_fen(),
                            })
                            .await;
                        self.last_move = mv.to_fen();
                    }
                } else {
                    let _ = self
                        .game_channel
                        .send(GameMessage::Resign(String::from("AI")))
                        .await;
                }
            }
            Err(_) => {}
        };
    }

    async fn redirect_to_placement(&mut self, message: &RedirectToPlacement) {
        let variant = Variant::from(message.variant);
        self.position.update_variant(variant);
        let _ = self.position.set_sfen(&message.sfen);
        let stm = self.position.side_to_move();
        if stm == self.player {
            let moves = self.position.get_placement_squares();

            let random_move: Vec<_> = moves.iter().collect();

            let Some((&key, value)) = random_move.choose(&mut rng()) else {
                let _ = self
                    .game_channel
                    .clone()
                    .send(GameMessage::Resign(String::from("AI")))
                    .await;
                return;
            };

            let piece_type = PieceType::try_from(key).unwrap();
            let piece = Piece {
                piece_type,
                color: self.player,
            };
            let value: Vec<_> = value.into_iter().collect();

            let Some(sq) = value.choose(&mut rng()) else {
                let _ = self
                    .game_channel
                    .send(GameMessage::Resign(String::from("AI")))
                    .await;
                return;
            };

            self.position.place(piece, *sq);

            let message = GameMessage::GameMove {
                player: "AI".to_string(),
                game_move: format!("{}@{}", piece.to_string(), sq.to_string()),
            };
            let _ = self.game_channel.send(message).await;
        }
    }

    async fn place_piece(&mut self, mv: Option<(Piece, S)>) {
        if let Some(mv) = mv {
            if mv.0.color == self.player {
                return;
            }
            let _ = self.position.place(mv.0, mv.1);
        }

        while self.position.side_to_move() == self.player {
            let moves = self.position.get_placement_squares();

            let random_move: Vec<_> = moves.iter().collect();

            let Some((&key, value)) = random_move.choose(&mut rng()) else {
                break;
            };

            let piece_type = PieceType::try_from(key).unwrap();
            let piece = Piece {
                piece_type,
                color: self.player,
            };
            let value: Vec<_> = value.into_iter().collect();

            let Some(sq) = value.choose(&mut rng()) else {
                break;
            };

            let mv = self.position.place(piece, *sq);
            if let Some(_) = mv {
                let message = GameMessage::GameMove {
                    player: "AI".to_string(),
                    game_move: format!("{}@{}", piece.to_string(), sq.to_string()),
                };
                let _ = self.game_channel.send(message).await;
            } else {
                break;
            }
        }
    }

    async fn next_stage(&mut self) {
        if self.position.get_hand(self.player, false) == ""
            && self.position.get_hand(self.player.flip(), false) == ""
            && !self.placement_finished
        {
            self.placement_finished = true;
            let sfen = self.position.generate_sfen();
            let _ = self.position.set_sfen(&sfen);

            if self.position.side_to_move() == self.player {
                self.move_piece("").await;
            }
        }
    }

    async fn join(&self, player_sender: Sender<WsMessage>) {
        time::sleep(time::Duration::from_secs(3)).await;
        let _ = self
            .game_channel
            .send(GameMessage::Join("AI".to_string(), player_sender.clone()))
            .await;
    }

    async fn select(&mut self) {
        let hand = self.get_pocket(self.selection.variant(), self.player);
        self.selection.set_hand(&hand);
        let hand = self.selection.to_sfen(self.player, true);
        for piece in hand.chars() {
            let _ = self
                .game_channel
                .send(GameMessage::GameMove {
                    player: String::from("AI"),
                    game_move: format!("+{}", piece),
                })
                .await;
        }
        let _ = self
            .game_channel
            .send(GameMessage::GameMove {
                player: String::from("AI"),
                game_move: "c".to_string(),
            })
            .await;
    }
}

pub struct Pockets {
    mini: Vec<String>,
    mini_fairy: Vec<String>,
    standard: Vec<String>,
    standard_fairy: Vec<String>,
    large: Vec<String>,
    large_fairy: Vec<String>,
}

impl Pockets {
    pub fn new() -> Self {
        let mini = env::var("MINI").unwrap_or("qnn".to_string());
        let mini = Self::add_items(mini);
        let mini_fairy = env::var("MINI_FAIRY").unwrap_or("qnn".to_string());
        let mini_fairy = Self::add_items(mini_fairy);
        let standard = env::var("STANDARD").unwrap_or("qnn".to_string());
        let standard = Self::add_items(standard);
        let standard_fairy = env::var("STANDARD_FAIRY").unwrap_or("qnn".to_string());
        let standard_fairy = Self::add_items(standard_fairy);
        let large = env::var("LARGE").unwrap_or("qnnr".to_string());
        let large = Self::add_items(large);
        let large_fairy = env::var("LARGE_FAIRY").unwrap_or("qnnrp".to_string());
        let large_fairy = Self::add_items(large_fairy);

        Self {
            mini,
            mini_fairy,
            standard,
            standard_fairy,
            large,
            large_fairy,
        }
    }

    fn add_items(pockets: String) -> Vec<String> {
        let pockets = pockets.split(",").collect::<Vec<&str>>();
        let mut v = vec![];
        for pocket in pockets {
            v.push(pocket.to_string());
        }
        v
    }
}
