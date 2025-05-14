use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, oneshot};

use crate::database::Database;
use crate::{database::redis::UserSession, AppState};

use super::channels::game::GameMessage;
use super::channels::game_requests::{GameRequest, GameRequestMessage};
use super::channels::games::GamesMessage;
use super::channels::message_types::MessageType;
use super::channels::players::PlayersMessage;
use super::channels::tv::TvMessage;
use super::channels::WsState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<UserAgent>>,
    State(state): State<AppState>,
    user: UserSession,
) -> impl IntoResponse {
    let headers = &user.headers();
    (
        headers.clone(),
        ws.on_upgrade(|socket| websocket(socket, state.db, state.ws, user)),
    )
}

async fn websocket(
    stream: WebSocket,
    db: Arc<Database>,
    ws: Arc<WsState>,
    session: UserSession,
) {
    let (mut sender, mut receiver) = stream.split();
    let (player_sender, mut player_recv) = mpsc::channel(20);

    let socket_send_task = tokio::spawn(async move {
        while let Some(WsMessage::Message(message)) = player_recv.recv().await {
            let _ = sender.send(Message::Text(message.into())).await;
        }
    });

    let _ = tokio::spawn(async move {
        let mut current_room = CurrentRoom::NoRoom;
        let mut current_game: Option<Sender<GameMessage>> = None;
        let _ = ws
            .players
            .send(PlayersMessage::Join {
                player: session.username.to_string(),
                sender: player_sender.clone(),
            })
            .await;
        while let Some(Ok(message)) = receiver.next().await {
            let Message::Text(message) = message else {
                socket_send_task.abort();
                break;
            };

            let Ok(value) = serde_json::from_str::<Value>(&message) else {
                continue;
            };

            let Ok(message) = serde_json::from_value::<ClientMessage>(value.clone())
            else {
                continue;
            };
            match message.t {
                MessageType::ChangeRoom => {
                    let Ok(new_room) = serde_json::from_value::<String>(message.d)
                    else {
                        continue;
                    };
                    let new_room = CurrentRoom::from(new_room);
                    if current_room == new_room {
                        continue;
                    }
                    match current_room {
                        CurrentRoom::NoRoom => {}
                        CurrentRoom::Home => {
                            let _ = ws
                                .players
                                .send(PlayersMessage::Leave {
                                    player: session.username.to_string(),
                                    disconnected: false,
                                })
                                .await;
                            let _ = ws
                                .game_requests
                                .send(GameRequestMessage::Leave(
                                    session.username.to_string(),
                                ))
                                .await;
                        }
                        CurrentRoom::Tv => {
                            let _ = ws
                                .tv
                                .send(TvMessage::Leave(session.username.to_string()))
                                .await;
                        }
                        CurrentRoom::Game(ref id) => {
                            if let Some(ref game) = current_game {
                                let _ = game
                                    .send(GameMessage::Leave(id.to_string()))
                                    .await;
                            }
                            current_game = None;
                        }
                    };
                    current_room = new_room;
                    match current_room {
                        CurrentRoom::NoRoom => {}
                        CurrentRoom::Home => {
                            let _ = ws
                                .game_requests
                                .send(GameRequestMessage::Join(
                                    session.username.to_string(),
                                    player_sender.clone(),
                                ))
                                .await;
                            let _ = ws
                                .players
                                .send(PlayersMessage::Join {
                                    player: session.username.to_string(),
                                    sender: player_sender.clone(),
                                })
                                .await;
                        }
                        CurrentRoom::Tv => {
                            let _ = ws
                                .tv
                                .send(TvMessage::Join(
                                    session.username.to_string(),
                                    player_sender.clone(),
                                ))
                                .await;
                        }
                        CurrentRoom::Game(ref game) => {
                            let (sender, receiver) = oneshot::channel();
                            let _ = ws
                                .games
                                .send(GamesMessage::GetChannel {
                                    sender,
                                    id: game.to_string(),
                                })
                                .await;

                            if let Ok(game) = receiver.await {
                                let _ = game
                                    .send(GameMessage::Join(
                                        session.username.to_string(),
                                        player_sender.clone(),
                                    ))
                                    .await;
                                current_game = Some(game);
                            } else {
                                current_game = None;
                            }
                        }
                    };
                }
                MessageType::AddGameRequest => {
                    if current_room != CurrentRoom::Home {
                        continue;
                    }
                    let Ok(game_request) =
                        serde_json::from_value::<GameRequest>(message.d)
                    else {
                        continue;
                    };
                    let game_request = GameRequestMessage::AddGameRequest {
                        caller: session.username.to_string(),
                        request: game_request,
                    };
                    let _ = ws.game_requests.send(game_request).await;
                }
                MessageType::GetHand => {
                    let Some(ref game) = current_game else {
                        continue;
                    };
                    let _ = game
                        .send(GameMessage::GetHand(session.username.to_string()))
                        .await;
                }
                MessageType::SelectMove
                | MessageType::PlacePiece
                | MessageType::MovePiece
                | MessageType::ConfirmSelection => {
                    let Some(ref game) = current_game else {
                        continue;
                    };
                    let Ok(game_move) = serde_json::from_value::<String>(message.d)
                    else {
                        continue;
                    };

                    let _ = game
                        .send(GameMessage::GameMove {
                            player: session.username.to_string(),
                            game_move,
                        })
                        .await;
                }
                MessageType::Draw => {
                    let Some(ref game) = current_game else {
                        continue;
                    };
                    let _ = game
                        .send(GameMessage::Draw(session.username.to_string()))
                        .await;
                }
                MessageType::Resign => {
                    let Some(ref game) = current_game else {
                        continue;
                    };
                    let _ = game
                        .send(GameMessage::Resign(session.username.to_string()))
                        .await;
                }
                MessageType::GetTv => {
                    if current_room != CurrentRoom::Tv {
                        continue;
                    }

                    let _ = ws
                        .tv
                        .send(TvMessage::GetTv(session.username.to_string()))
                        .await;
                }
                MessageType::SaveState => {
                    if &session.username != &db.mod1 {
                        continue;
                    }
                }
                _ => {}
            }
        }

        let _ = ws
            .players
            .send(PlayersMessage::Leave {
                player: session.username.to_string(),
                disconnected: true,
            })
            .await;

        match current_room {
            CurrentRoom::NoRoom => {}
            CurrentRoom::Home => {
                let _ = ws
                    .game_requests
                    .send(GameRequestMessage::Leave(session.username.to_string()))
                    .await;
            }
            CurrentRoom::Tv => {
                let _ = ws
                    .tv
                    .send(TvMessage::Leave(session.username.to_string()))
                    .await;
            }
            CurrentRoom::Game(ref id) => {
                if let Some(ref game) = current_game {
                    let _ = game.send(GameMessage::Leave(id.to_string())).await;
                }
            }
        };

        socket_send_task.abort();
    });
}

#[derive(Serialize, Deserialize)]
pub struct ClientMessage {
    pub t: MessageType,
    pub d: Value,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CurrentRoom {
    NoRoom,
    Home,
    Tv,
    Game(String),
}

impl From<String> for CurrentRoom {
    fn from(value: String) -> Self {
        if value == "home" {
            return Self::Home;
        } else if value == "tv" {
            return Self::Tv;
        } else if value.starts_with("/game/") {
            let mut game_id = value.split("/game/");
            game_id.next();
            let game_id = game_id.next().unwrap_or_default();
            return Self::Game(game_id.to_string());
        }

        Self::NoRoom
    }
}

impl CurrentRoom {}

#[derive(Clone)]
pub enum WsMessage {
    Message(String),
}
