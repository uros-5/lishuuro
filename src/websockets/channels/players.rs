use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Sender};

use crate::websockets::handler::WsMessage;

use super::{
    message_types::MessageType,
    watchers::{SendTo, Watchers},
    WsState,
};

pub enum PlayersMessage {
    Join {
        player: String,
        sender: Sender<WsMessage>,
    },
    Leave {
        player: String,
        disconnected: bool,
    },

    SetWs(Arc<WsState>),
    Redirect {
        game: String,
        player: String,
    },
}

pub async fn players_task() -> Sender<PlayersMessage> {
    let (sender, mut recv) = mpsc::channel(1024);
    let mut watchers = Watchers::new();
    let mut names = HashSet::new();
    let _ = tokio::spawn(async move {
        let mut _ws = Arc::new(WsState::empty());
        while let Some(message) = recv.recv().await {
            match message {
                PlayersMessage::Join { player, sender } => {
                    watchers.add_watcher(player.to_string(), sender);
                    if !names.contains(&player) {
                        names.insert(player.to_string());
                    }
                    let msg = PlayersCount {
                        count: names.len() as u64,
                        t: MessageType::PlayerCount,
                    };
                    let msg = serde_json::json!(msg);
                    watchers
                        .notify(
                            WsMessage::Message(msg.to_string()),
                            super::watchers::SendTo::Everyone,
                        )
                        .await;
                }
                PlayersMessage::Leave {
                    player,
                    disconnected,
                } => {
                    watchers.remove_watcher(&player);
                    if disconnected {
                        names.remove(&player);
                        let msg = PlayersCount {
                            count: names.len() as u64,
                            t: MessageType::PlayerCount,
                        };
                        let msg = serde_json::json!(msg);
                        watchers
                            .notify(
                                WsMessage::Message(msg.to_string()),
                                SendTo::Everyone,
                            )
                            .await;
                    }
                }
                PlayersMessage::SetWs(ws_state) => _ws = ws_state,
                PlayersMessage::Redirect { game, player } => {
                    let msg = RedirectPlayer {
                        t: MessageType::RedirectToGame,
                        game,
                    };
                    let msg = serde_json::json!(msg).to_string();

                    watchers
                        .notify(
                            WsMessage::Message(msg),
                            SendTo::Players {
                                list: vec![player],
                                to_others: false,
                            },
                        )
                        .await;
                }
            };
        }
    });

    sender
}

#[derive(Serialize, Deserialize)]
pub struct PlayersCount {
    count: u64,
    t: MessageType,
}

#[derive(Serialize, Deserialize)]
pub struct RedirectPlayer {
    t: MessageType,
    game: String,
}
