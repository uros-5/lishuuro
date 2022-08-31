use serde::{Deserialize, Serialize};
use shuuro::position::Outcome;

use super::{MessageHandler, WsState};

#[derive(Clone, Serialize, Deserialize)]
pub struct GameGet {
    pub t: String,
    pub game_id: String,
    #[serde(default)]
    pub game_move: String,
}

pub enum LiveGameMove {
    BuyMove([bool; 2]),
    LostOnTime(usize),
    FirstMoveError(usize),
    PlaceMove(String, [u64; 2], bool, bool, [String; 2]),
    FightMove(String, [u64; 2], i32, String, [String; 2], String),
}
