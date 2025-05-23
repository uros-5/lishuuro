use std::{collections::HashMap, sync::Arc};

use shuuro::{
    attacks::Attacks,
    shuuro12::{
        attacks12::Attacks12, bitboard12::BB12, position12::P12, square12::Square12,
    },
    shuuro6::{attacks6::Attacks6, bitboard6::BB6, position6::P6, square6::Square6},
    shuuro8::{attacks8::Attacks8, bitboard8::BB8, position8::P8, square8::Square8},
    Variant,
};
use shuuro_engine::{
    engine12::search::{Defs12, Engine12},
    engine6::search::{Defs6, Engine6},
    engine8::search::{Defs8, Engine8},
};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

use crate::database::{clock::queries::unfinished, model::ShuuroGame, Database};

use super::{
    game::{game_task, GameMessage},
    game_requests::{GameRequest, GameRequestMessage},
    tv::TvMessage,
    WsState,
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
    SaveState,
}

pub async fn games_task(
    db: Arc<Database>,
    _: mpsc::Sender<TvMessage>,
) -> mpsc::Sender<GamesMessage> {
    let (sender, mut recv) = mpsc::channel(64);
    init();

    let _ = tokio::spawn(async move {
        let mut ws = Arc::new(WsState::empty());
        let mut channels = HashMap::new();
        let mut shutdown = false;
        while let Some(message) = recv.recv().await {
            //
            match message {
                GamesMessage::InsertGame { channel, id } => {
                    if shutdown == true {
                        continue;
                    }
                    channels.insert(id, channel);
                    let _ = ws.game_requests.send(GameRequestMessage::NewGame).await;
                }
                GamesMessage::RemoveGame { id } => {
                    channels.remove(&id);
                    if shutdown && channels.len() == 0 {
                        std::process::exit(1);
                    }
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

                    let games = unfinished(&db.mongo.games).await;
                    for game in games {
                        let variant = game.1.variant;
                        let request = GameRequest::empty();
                        match variant {
                            Variant::Shuuro | Variant::ShuuroFairy => {
                                game_task::<
                                    Square12,
                                    BB12<Square12>,
                                    Attacks12<Square12, BB12<Square12>>,
                                    P12<Square12, BB12<Square12>>,
                                    Engine12,
                                    Defs12,
                                    12,
                                    144,
                                    11,
                                >(
                                    db.clone(),
                                    ws.clone(),
                                    request,
                                    game.1._id.to_string(),
                                    String::new(),
                                    Some(game.1),
                                )
                                .await;
                            }
                            Variant::ShuuroMini | Variant::ShuuroMiniFairy => {
                                game_task::<
                                    Square6,
                                    BB6<Square6>,
                                    Attacks6<Square6, BB6<Square6>>,
                                    P6<Square6, BB6<Square6>>,
                                    Engine6,
                                    Defs6,
                                    6,
                                    36,
                                    4,
                                >(
                                    db.clone(),
                                    ws.clone(),
                                    request,
                                    game.1._id.to_string(),
                                    String::new(),
                                    Some(game.1),
                                )
                                .await;
                            }
                            Variant::Standard | Variant::StandardFairy => {
                                game_task::<
                                    Square8,
                                    BB8<Square8>,
                                    Attacks8<Square8, BB8<Square8>>,
                                    P8<Square8, BB8<Square8>>,
                                    Engine8,
                                    Defs8,
                                    8,
                                    64,
                                    7,
                                >(
                                    db.clone(),
                                    ws.clone(),
                                    request,
                                    game.1._id.to_string(),
                                    String::new(),
                                    Some(game.1),
                                )
                                .await;
                            }
                        };
                    }
                }
                GamesMessage::GetGame(sender, ref id) => {
                    if let Some(channel) = channels.get(id) {
                        let _ = channel.send(GameMessage::GetGame(sender)).await;
                    }
                }
                GamesMessage::SaveState => {
                    shutdown = true;
                    for channel in &channels {
                        let _ = channel.1.send(GameMessage::SaveState).await;
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
