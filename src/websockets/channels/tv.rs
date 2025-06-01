use std::{collections::HashMap, ops::Deref, sync::Arc};
use typeshare::typeshare;

use serde::Serialize;
use tokio::sync::mpsc::{self, Sender};

use crate::websockets::handler::WsMessage;

use super::{
    game::RedirectToPlacement,
    message_types::MessageType,
    watchers::{SendTo, Watchers},
};

pub enum TvMessage {
    Leave(String),
    Join(String, Sender<WsMessage>),
    Add(Arc<RedirectToPlacement>),
    Move {
        id: String,
        sfen: String,
        first_move_error: bool,
    },
    Remove {
        id: String,
    },
    GetTv(String),
}

pub async fn tv_task() -> mpsc::Sender<TvMessage> {
    let (sender, mut recv) = mpsc::channel(64);
    let mut watchers = Watchers::new();
    let mut tv = HashMap::with_capacity(9);
    let sender2 = sender.clone();

    let _ = tokio::spawn(async move {
        while let Some(message) = recv.recv().await {
            match message {
                TvMessage::Join(player, sender) => {
                    watchers.add_watcher(player, sender);
                }
                TvMessage::Leave(ref player) => {
                    watchers.remove_watcher(player);
                }
                TvMessage::Add(placement) => {
                    if tv.len() < 10 {
                        let placement = placement.deref();
                        tv.insert(placement.id.to_owned(), placement.clone());
                        let message = NewTvGame {
                            t: MessageType::AddTvGame,
                            game: placement.clone(),
                        };
                        watchers
                            .notify(
                                WsMessage::Message(
                                    serde_json::json!(message).to_string(),
                                ),
                                SendTo::Everyone,
                            )
                            .await;
                    }
                }
                TvMessage::Move {
                    ref id,
                    sfen,
                    first_move_error,
                } => {
                    if let Some(game) = tv.get_mut(id) {
                        game.sfen = sfen.to_string();
                        let message = NewTvMove {
                            t: MessageType::NewTvMove,
                            game: id.to_string(),
                            game_move: sfen,
                        };
                        let message = serde_json::json!(message).to_string();
                        watchers
                            .notify(WsMessage::Message(message), SendTo::Everyone)
                            .await;
                        if first_move_error {
                            let _ = sender2
                                .send(TvMessage::Remove { id: id.to_string() })
                                .await;
                        }
                    }
                }
                TvMessage::Remove { ref id } => {
                    tv.remove(id);
                    let message = RemoveTvGame {
                        t: MessageType::RemoveTVGame,
                        game: id.to_string(),
                    };
                    watchers
                        .notify(
                            WsMessage::Message(
                                serde_json::json!(message).to_string(),
                            ),
                            SendTo::Everyone,
                        )
                        .await;
                }
                TvMessage::GetTv(player) => {
                    let mut games = vec![];
                    for game in &tv {
                        games.push(game.1);
                    }
                    let message = AllGames {
                        t: MessageType::GetTv,
                        games,
                    };
                    watchers
                        .notify(
                            WsMessage::Message(
                                serde_json::json!(message).to_string(),
                            ),
                            SendTo::Players {
                                list: vec![player],
                                to_others: false,
                            },
                        )
                        .await;
                }
            }
        }
    });
    sender.clone()
}

#[derive(Serialize)]
#[typeshare]
struct NewTvGame {
    t: MessageType,
    game: RedirectToPlacement,
}

#[derive(Serialize)]
#[typeshare]
struct RemoveTvGame {
    t: MessageType,
    game: String,
}

#[derive(Serialize)]
#[typeshare]
struct NewTvMove {
    t: MessageType,
    game: String,
    game_move: String,
}

#[derive(Serialize)]
#[typeshare]
struct AllGames<'a> {
    t: MessageType,
    games: Vec<&'a RedirectToPlacement>,
}
