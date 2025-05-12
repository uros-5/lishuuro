use std::{collections::HashMap, sync::Arc};

use shuuro::{
    attacks::Attacks, shuuro12::attacks12::Attacks12, shuuro6::attacks6::Attacks6,
    shuuro8::attacks8::Attacks8,
};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

use crate::database::{model::ShuuroGame, Database};

use super::{
    game::GameMessage, game_reguests::GameRequestMessage, tv::TvMessage, WsState,
};

pub enum GamesMessage {
    GetChannel {
        sender: oneshot::Sender<mpsc::Sender<GameMessage>>,
        id: String,
    },
    SetWs(Arc<WsState>),
    InsertGame {
        channel: Sender<GameMessage>,
        id: String,
    },
    RemoveGame {
        id: String,
    },
    GetGame(oneshot::Sender<ShuuroGame>, String),
}

pub async fn games_task(
    _: Arc<Database>,
    _: mpsc::Sender<TvMessage>,
) -> mpsc::Sender<GamesMessage> {
    let (sender, mut recv) = mpsc::channel(64);
    init();
    let _ = tokio::spawn(async move {
        let mut ws = Arc::new(WsState::empty());
        let mut channels = HashMap::new();
        while let Some(message) = recv.recv().await {
            //
            match message {
                GamesMessage::InsertGame { channel, id } => {
                    channels.insert(id, channel);
                    let _ = ws.game_requests.send(GameRequestMessage::NewGame).await;
                }
                GamesMessage::RemoveGame { id } => {
                    channels.remove(&id);
                }
                GamesMessage::GetChannel { sender, id } => {
                    let Some(game) = channels.get(&id) else {
                        drop(sender);
                        continue;
                    };
                    let _ = sender.send(game.clone());
                }
                GamesMessage::SetWs(ws_state) => {
                    ws = ws_state;
                }
                GamesMessage::GetGame(sender, ref id) => {
                    if let Some(channel) = channels.get(id) {
                        let _ = channel.send(GameMessage::GetGame(sender)).await;
                    }
                }
            }
        }
    });
    sender.clone()
}

fn init() {
    Attacks12::init();
    Attacks6::init();
    Attacks8::init();
}
