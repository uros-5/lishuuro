use crate::database::serde_helpers::deserialize_color;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shuuro::{
    shuuro12::{
        attacks12::Attacks12, bitboard12::BB12, position12::P12, square12::Square12,
    },
    shuuro6::{attacks6::Attacks6, bitboard6::BB6, position6::P6, square6::Square6},
    shuuro8::{attacks8::Attacks8, bitboard8::BB8, position8::P8, square8::Square8},
    Color,
};
use shuuro::{SubVariant, Variant};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::{
    database::{
        clock::queries::game_id,
        serde_helpers::{deserialize_subvariant, deserialize_variant},
        Database,
    },
    websockets::handler::WsMessage,
};

use super::{
    game::game_task,
    message_types::MessageType,
    watchers::{SendTo, Watchers},
    WsState,
};

pub const VARIANTS: [&str; 6] = [
    "shuuro",
    "shuuroFairy",
    "standard",
    "standardFairy",
    "shuuroMini",
    "shuuroMiniFairy",
];
pub const DURATION_RANGE: [i64; 28] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 25, 30,
    35, 40, 45, 60, 75, 90,
];

pub async fn game_requests_task(
    db: Arc<Database>,
) -> mpsc::Sender<GameRequestMessage> {
    let (sender, mut recv) = mpsc::channel(64);
    let _ = tokio::spawn(async move {
        let mut watchers = Watchers::new();
        let mut playing = HashSet::new();
        let mut ws = Arc::new(WsState::empty());
        let mut games_count = 0;
        while let Some(message) = recv.recv().await {
            match message {
                GameRequestMessage::AddGameRequest { caller, request } => {
                    if playing.contains(&caller) {
                        continue;
                    }
                    if playing.len() >= 60 {
                        continue;
                    }
                    match &request.game_type {
                        TypeOfGame::VsFriend(ref friend) => {
                            if friend == &caller {
                                continue;
                            } else if playing.contains(friend) {
                                continue;
                            }

                            playing.insert(caller.to_string());
                            match request.variant {
                                Variant::Shuuro | Variant::ShuuroFairy => {
                                    game_task::<
                                        Square12,
                                        BB12<Square12>,
                                        Attacks12<Square12, BB12<Square12>>,
                                        P12<Square12, BB12<Square12>>,
                                    >(
                                        db.clone(),
                                        ws.clone(),
                                        request,
                                        game_id(&db.mongo.games).await,
                                        caller,
                                    )
                                    .await;
                                }
                                Variant::ShuuroMini | Variant::ShuuroMiniFairy => {
                                    game_task::<
                                        Square6,
                                        BB6<Square6>,
                                        Attacks6<Square6, BB6<Square6>>,
                                        P6<Square6, BB6<Square6>>,
                                    >(
                                        db.clone(),
                                        ws.clone(),
                                        request,
                                        game_id(&db.mongo.games).await,
                                        caller,
                                    )
                                    .await;
                                }
                                Variant::Standard | Variant::StandardFairy => {
                                    game_task::<
                                        Square8,
                                        BB8<Square8>,
                                        Attacks8<Square8, BB8<Square8>>,
                                        P8<Square8, BB8<Square8>>,
                                    >(
                                        db.clone(),
                                        ws.clone(),
                                        request,
                                        game_id(&db.mongo.games).await,
                                        caller,
                                    )
                                    .await;
                                }
                            };
                        }
                        TypeOfGame::VsAi(_level) => {
                            continue;
                        }
                    };
                }
                GameRequestMessage::RedirectToGame => todo!(),
                GameRequestMessage::Join(player, sender) => {
                    watchers.add_watcher(player, sender);
                    let msg = GamesCount {
                        t: MessageType::GameCount,
                        count: games_count,
                    };

                    watchers
                        .notify(
                            WsMessage::Message(json!(msg).to_string()),
                            SendTo::Everyone,
                        )
                        .await;
                }
                GameRequestMessage::Leave(player) => {
                    watchers.remove_watcher(&player);
                }
                GameRequestMessage::SetWs(ws_state) => ws = ws_state,
                GameRequestMessage::RemovePlayers(players) => {
                    for i in players {
                        playing.remove(&i);
                    }
                    games_count -= 1;
                    let msg = GamesCount {
                        t: MessageType::GameCount,
                        count: games_count,
                    };
                    watchers
                        .notify(
                            WsMessage::Message(json!(msg).to_string()),
                            SendTo::Everyone,
                        )
                        .await;
                }
                GameRequestMessage::NewGame => {
                    games_count += 1;
                    let msg = GamesCount {
                        t: MessageType::GameCount,
                        count: games_count,
                    };

                    watchers
                        .notify(
                            WsMessage::Message(json!(msg).to_string()),
                            SendTo::Everyone,
                        )
                        .await;
                }
                GameRequestMessage::AddActivePlayer(player) => {
                    playing.insert(player);
                }
            }
        }
    });
    sender
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all(serialize = "snake_case"))]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum TypeOfGame {
    VsFriend(String),
    VsAi(u8),
}

#[derive(Clone)]
pub enum GameRequestMessage {
    AddGameRequest {
        caller: String,
        request: GameRequest,
    },
    Join(String, Sender<WsMessage>),
    Leave(String),
    RedirectToGame,
    SetWs(Arc<WsState>),
    RemovePlayers([String; 2]),
    AddActivePlayer(String),
    NewGame,
}

#[derive(Clone, Deserialize, PartialEq, Eq, Debug)]
pub struct GameRequest {
    pub minutes: i64,
    pub incr: i64,
    #[serde(serialize_with = "serialize_variant")]
    #[serde(deserialize_with = "deserialize_variant")]
    pub variant: Variant,
    #[serde(serialize_with = "serialize_subvariant")]
    #[serde(deserialize_with = "deserialize_subvariant")]
    pub sub_variant: Option<SubVariant>,
    #[serde(deserialize_with = "deserialize_color")]
    color: Color,
    pub game_type: TypeOfGame,
}

impl GameRequest {
    pub fn is_valid(&self) -> bool {
        if VARIANTS.contains(&self.variant.to_string().as_str())
            && DURATION_RANGE.contains(&self.minutes)
            && (DURATION_RANGE.contains(&self.incr) || self.incr == 0)
        {
            return true;
        }
        false
    }

    pub fn colors(&self, player: &String, other: &String) -> [String; 2] {
        let mut color = self.color;
        if self.color == Color::NoColor {
            color = self.random_color();
        }
        if color == Color::White {
            [String::from(player), String::from(other)]
        } else {
            [String::from(other), String::from(player)]
        }
    }

    fn random_color(&self) -> Color {
        if rand::random() {
            Color::White
        } else {
            Color::Black
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GamesCount {
    t: MessageType,
    count: u64,
}
