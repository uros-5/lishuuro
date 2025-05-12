use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::websockets::handler::WsMessage;

#[derive(Clone)]
pub struct Watchers {
    pub players: HashMap<String, Vec<mpsc::Sender<WsMessage>>>,
}

impl Default for Watchers {
    fn default() -> Self {
        Self::new()
    }
}

impl Watchers {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }
    pub fn add_watcher(
        &mut self,
        player: String,
        sender: mpsc::Sender<WsMessage>,
    ) -> bool {
        if let Some(sockets) = self.players.get_mut(&player) {
            let old_sender = sockets.iter().find(|item| item.same_channel(&sender));
            if old_sender.is_none() {
                sockets.push(sender);
            }
            return false;
        }
        self.players.insert(player, vec![sender]).is_none()
    }

    pub fn remove_watcher(
        &mut self,
        player: &String,
    ) -> Option<Vec<mpsc::Sender<WsMessage>>> {
        self.players.remove(player)
    }

    pub async fn notify(&self, message: WsMessage, send_to: SendTo) {
        match send_to {
            SendTo::Everyone => {
                for player in &self.players {
                    for socket in player.1 {
                        socket.send(message.clone()).await.ok();
                    }
                }
            }
            SendTo::Players { list, to_others } => {
                for player in &list {
                    if let Some(player) = self.players.get(player) {
                        for socket in player {
                            socket.send(message.clone()).await.ok();
                        }
                    }
                }
                if to_others {
                    for player in &self.players {
                        if list.contains(player.0) {
                            continue;
                        }
                        for socket in player.1 {
                            socket.send(message.clone()).await.ok();
                        }
                    }
                }
            }
        }
    }
}

pub enum SendTo {
    Everyone,
    Players { list: Vec<String>, to_others: bool },
}
