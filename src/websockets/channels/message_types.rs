use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Debug)]
#[repr(u8)]
pub enum MessageType {
    ChangeRoom,
    AddGameRequest,
    RedirectToGame,
    PlayerCount,
    GameCount,
    // game
    StartClock,
    GetGame,
    GetConfirmed,
    GetHistory,
    GetHand,
    SelectMove,
    PlacePiece,
    MovePiece,
    Draw,
    Resign,
    GameEnd,
    // tv
    GetTv,
    AddTvGame,
    NewTvMove,
    RemoveTVGame,
    SaveState,
    ReloadJinja,
    ConfirmSelection,
}

impl MessageType {
    pub fn into_number(&self) -> usize {
        *self as usize
    }
}
