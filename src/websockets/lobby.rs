use super::messages::{Connect, Disconnect, RegularMessage, WsMessage};
use crate::models::live_games::LiveGames;
use crate::models::model::{ActivePlayer, ChatItem, LobbyGame, LobbyGames, ShuuroGame, User};
use actix::prelude::{Actor, Context, Handler, Recipient};
use actix::AsyncContext;
use actix::WrapFuture;
use mongodb::Collection;
use serde_json;
use std::collections::HashMap;

type Socket = Recipient<WsMessage>;

#[derive(Clone)]
pub struct Lobby {
    pub chat: Vec<ChatItem>,
    pub active_players: HashMap<ActivePlayer, Socket>,
    pub games: LiveGames,
    pub lobby: LobbyGames,
    pub db_users: Collection<User>,
    pub db_shuuro_games: Collection<ShuuroGame>,
}

impl Lobby {
    pub fn new(db_users: Collection<User>, db_shuuro_games: Collection<ShuuroGame>) -> Self {
        Lobby {
            chat: vec![],
            active_players: HashMap::new(),
            games: LiveGames::default(),
            lobby: LobbyGames::default(),
            db_users,
            db_shuuro_games,
        }
    }

    pub fn send_message(&self, player: &ActivePlayer, message: serde_json::Value) {
        if let Some(socket_recipient) = self.active_players.get(player) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned().to_string()));
        } else {
        }
    }

    pub fn send_message_to_all(&self, message: serde_json::Value) {
        for user in self.active_players.iter() {
            user.1.do_send(WsMessage(message.to_owned().to_string()));
        }
    }

    pub fn send_message_to_selected(&self, message: serde_json::Value, users: [String; 2]) {
        for user in self.active_players.iter() {
            if users.contains(&&user.0.username()) {
                user.1.do_send(WsMessage(message.to_owned().to_string()));
            }
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<RegularMessage> for Lobby {
    type Result = ();
    //type Result = Future;
    fn handle(&mut self, msg: RegularMessage, ctx: &mut Context<Self>) -> Self::Result {
        let data = serde_json::from_str::<serde_json::Value>(&msg.text);
        let mut res: serde_json::Value = serde_json::json!({"t": "error"});
        match data {
            Ok(i) => {
                let data_type = &i["t"];
                match data_type {
                    serde_json::Value::String(t) => {
                        if t == "home_chat_full" {
                            res = serde_json::json!({"t": t, "lines": self.chat});
                        } else if t == "active_players_count" {
                            res = serde_json::json!({"t": t, "cnt": self.active_players.len()});
                        } else if t == "active_matches_count" {
                            res = serde_json::json!({"t": t, "cnt": self.games.shuuro_games.len()});
                        } else if t == "home_chat_message" {
                            let m = serde_json::from_str::<ChatItem>(&msg.text);
                            if let Ok(mut m) = m {
                                m.update(&msg.player.username());
                                if m.message.len() > 0 && m.message.len() < 50 {
                                    res = m.response();
                                    self.chat.push(m);
                                    return self.send_message_to_all(res);
                                }
                            }
                        } else if t == "home_lobby_full" {
                            res = self.lobby.response()
                        } else if t == "home_lobby_add" {
                            let m = serde_json::from_str::<LobbyGame>(&msg.text);
                            if let Ok(mut game) = m {
                                if game.is_valid() {
                                    if self.lobby.can_add(&game) {
                                        res = game.response(&t);
                                        self.lobby.add(game);
                                        return self.send_message_to_all(res);
                                    }
                                }
                            }
                        } else if t == "home_lobby_accept" {
                            let m = serde_json::from_str::<LobbyGame>(&msg.text);
                            if let Ok(mut game) = m {
                                if game.is_valid() {
                                    if &game.username() == &msg.player.username() {
                                        res = game.response(&String::from("home_lobby_remove"));
                                        let deleted = self.lobby.delete(game);
                                        if deleted >= 0 {
                                            return self.send_message_to_all(res);
                                        }
                                        res = serde_json::json!({"t": "error"});
                                        return self.send_message_to_all(res);
                                    } else {
                                        let users = [game.username(), msg.player.username()];
                                        let shuuro_game = ShuuroGame::from(&game);
                                        let res = game.response(&String::from("home_lobby_remove"));
                                        let deleted = self.lobby.delete(game);
                                        if deleted >= 0 {
                                            self.send_message_to_all(res);
                                        }
                                        let db_shuuro_games = self.db_shuuro_games.clone();
                                        let self_2 = self.clone();
                                        let b = Box::pin(async move {
                                            let game_added =
                                                db_shuuro_games.insert_one(shuuro_game, None);
                                            match game_added.await {
                                                _ => {
                                                    let res =
                                                        serde_json::json!({"t": "redirect_match"});
                                                    return self_2
                                                        .send_message_to_selected(res, users);
                                                }
                                            }
                                        });
                                        let actor_future = b.into_actor(self);
                                        ctx.spawn(actor_future);
                                    }
                                }
                            }
                        } else {
                            () //res = serde_json::json!({"t": "error"});
                        }
                    }
                    _ => {
                        () //res = serde_json::json!({"t": "error"});
                    }
                }
            }
            Err(_err) => {
                () //res = serde_json::json!({"t": "error"});
            }
        }

        self.send_message(&msg.player, res)
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let user = self.active_players.get(&msg.player);
        match user {
            Some(_i) => {
                ();
            }
            None => {
                let player = msg.player.clone();
                self.active_players.insert(msg.player, msg.addr);
                self.send_message(
                    &player.clone(),
                    serde_json::json!({"t": "connected","msg": "User connected"}),
                );
            }
        }
        for player in self.active_players.iter() {
            self.send_message(
                &player.0.clone(),
                serde_json::json!({"t": "home_chat_full", "lines": self.chat}),
            );
            self.send_message(
                &player.0.clone(),
                serde_json::json!({"t": "active_players_count", "cnt": self.active_players.len()}),
            );
            self.send_message(
                &player.0.clone(),
                serde_json::json!({"t": "active_matches_count", "cnt": self.games.shuuro_games.len()}),
            );
        }
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Context<Self>) {
        self.active_players.remove(&msg.player);
        self.lobby.delete_by_user(&msg.player);
        let player_count =
            serde_json::json!({"t": "active_players_count", "cnt": self.active_players.len()});
        let matches_count =
            serde_json::json!({"t": "active_matches_count", "cnt": self.games.shuuro_games.len()});
        self.send_message_to_all(player_count);
        self.send_message_to_all(matches_count);
    }
}
