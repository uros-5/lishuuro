use std::sync::Arc;

use game_requests::{game_requests_task, GameRequestMessage};
use games::{games_task, GamesMessage};
use jinja::JinjaMessage;
use players::{players_task, PlayersMessage};
use tokio::sync::mpsc;
use tv::{tv_task, TvMessage};

use crate::database::Database;

pub mod ai;
pub mod chat;
pub mod clock;
pub mod game;
pub mod game_requests;
pub mod games;
pub mod jinja;
pub mod message_types;
pub mod player;
pub mod players;
pub mod tv;
pub mod watchers;

#[derive(Clone)]
pub struct WsState {
    pub game_requests: mpsc::Sender<GameRequestMessage>,
    pub tv: mpsc::Sender<TvMessage>,
    pub games: mpsc::Sender<GamesMessage>,
    pub players: mpsc::Sender<PlayersMessage>,
    pub jinja: mpsc::Sender<JinjaMessage>,
}

impl WsState {
    pub async fn new(db: Arc<Database>) -> Self {
        let tv = tv_task().await;
        let games = games_task(db.clone(), tv.clone()).await;
        let game_requests = game_requests_task(db.clone()).await;
        let players = players_task().await;
        let jinja = mpsc::channel(200).0;

        Self {
            tv,
            games,
            game_requests,
            players,
            jinja,
        }
    }

    pub async fn send_ws(&self, ws: Arc<WsState>) {
        let _ = self
            .game_requests
            .send(GameRequestMessage::SetWs(ws.clone()))
            .await;
        let _ = self.players.send(PlayersMessage::SetWs(ws.clone())).await;
        let _ = self.games.send(GamesMessage::SetWs(ws.clone())).await;
    }

    pub fn empty() -> Self {
        Self {
            game_requests: mpsc::channel(2).0,
            tv: mpsc::channel(2).0,
            games: mpsc::channel(2).0,
            players: mpsc::channel(2).0,
            jinja: mpsc::channel(2).0,
        }
    }
}
