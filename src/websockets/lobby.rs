use super::messages::{
    Connect, Disconnect, GameMessage, GameMessageType, RegularMessage, WsMessage,
};
use crate::models::live_games::LiveGames;
use crate::models::model::{
    ActivePlayer, ChatItem, GameGetConfirmed, GameGetHand, GameMove, GameRequest, LobbyGame,
    LobbyGames, NewsItem, ShuuroGame, User,
};
use actix::prelude::{Actor, Context, Handler, Recipient};
use actix::AsyncContext;
use actix::WrapFuture;
use bson::{doc, oid::ObjectId};
use futures::stream::TryStreamExt;
use futures::Future;
use mongodb::Collection;
use serde_json;
use std::collections::HashMap;
use std::str::FromStr;

type Socket = Recipient<WsMessage>;

#[derive(Clone)]
pub struct Lobby {
    pub chat: Vec<ChatItem>,
    pub active_players: HashMap<ActivePlayer, Socket>,
    pub games: LiveGames,
    pub lobby: LobbyGames,
    pub db_users: Collection<User>,
    pub db_shuuro_games: Collection<ShuuroGame>,
    pub news: Collection<NewsItem>,
    pub counter: i32,
}

impl Lobby {
    pub fn new(
        db_users: Collection<User>,
        db_shuuro_games: Collection<ShuuroGame>,
        news: Collection<NewsItem>,
    ) -> Self {
        Lobby {
            chat: vec![],
            active_players: HashMap::new(),
            games: LiveGames::default(),
            lobby: LobbyGames::default(),
            db_users,
            db_shuuro_games,
            news,
            counter: 0,
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
    pub fn update_entire_game(
        &'static self,
        id: &'_ String,
        game: &'_ ShuuroGame,
    ) -> impl Future<Output = ()> + 'static {
        let filter = doc! {"_id": ObjectId::from_str(id.as_str()).unwrap()};
        let update = doc! {"$set": bson::to_bson(&game).unwrap()};
        let b = Box::pin(async move {
            let game1 = self
                .db_shuuro_games
                .find_one_and_update(filter, update, None);
            match game1.await {
                _g => {}
            };
        });
        b
    }

    fn update_spectator(&mut self, player: &ActivePlayer, watches: &str) {
        let key = self.active_players.get_key_value(player);
        if let Some(p) = key {
            let s = p.1.clone();
            let mut new_player = player.clone();
            new_player.update_watches(watches);
            self.active_players.remove(player);
            self.active_players.insert(new_player, s);
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
                        } else if t == "active_games_count" {
                            println!("{}", &msg.player.watches());
                            res = serde_json::json!({"t": t, "cnt": self.games.shuuro_games.len()});
                        } else if t == "home_news" {
                            let ctx2 = ctx.address().clone();
                            let news = self.news.clone();
                            let active_player = msg.player.clone();
                            let b = Box::pin(async move {
                                let all = news.find(doc! {}, None).await;
                                if let Ok(c) = all {
                                    let news_: Vec<NewsItem> =
                                        c.try_collect().await.unwrap_or_else(|_| vec![]);
                                    ctx2.do_send(GameMessage {
                                        message_type: GameMessageType::news(active_player, news_),
                                    });
                                    //ctx2.send_message(&msg.player, res)
                                }
                            });
                            let actor_future = b.into_actor(self);
                            ctx.spawn(actor_future);
                        } else if t == "live_game_start" {
                            let m = serde_json::from_str::<GameRequest>(&msg.text);
                            if let Ok(m) = m {
                                let game = self.games.get_game(&m.game_id);
                                match game {
                                    Some(g) => {
                                        res = serde_json::json!({"t": "live_game_start", "game_id": &g.0.clone(), "game_info": &g.1});
                                    }
                                    None => (),
                                }
                            }
                        } else if t == "live_game_buy" || t == "live_game_confirm" {
                            let m = serde_json::from_str::<GameMove>(&msg.text);
                            if let Ok(m) = m {
                                self.games
                                    .buy(&m.game_id, m.game_move, &msg.player.username());
                                // if both sides are confirmed then notify them and redirect players.
                                if !self.games.confirmed_players(&m.game_id).contains(&false) {
                                    res = self.games.set_deploy(&m.game_id);
                                    let res2 = serde_json::json!({"t": "pause_confirmed", "confirmed": &self.games.confirmed_players(&m.game_id)});
                                    self.send_message_to_selected(
                                        res2,
                                        self.games.players(&m.game_id),
                                    );
                                    return self.send_message_to_selected(
                                        res,
                                        self.games.players(&m.game_id),
                                    );
                                } else if t == "live_game_confirm" {
                                    res = serde_json::json!({"t": "pause_confirmed", "confirmed": &self.games.confirmed_players(&m.game_id)});
                                    return self.send_message_to_selected(
                                        res,
                                        self.games.players(&m.game_id),
                                    );
                                } else {
                                    return ();
                                }
                            }
                        } else if t == "live_game_place" {
                            let m = serde_json::from_str::<GameMove>(&msg.text);
                            if let Ok(m) = m {
                                let placed = self.games.place(
                                    &m.game_id,
                                    m.game_move,
                                    &msg.player.username(),
                                );
                                if let Some(mut placed) = placed {
                                    *placed.get_mut("game_id").unwrap() =
                                        serde_json::json!(m.game_id);
                                    self.send_message_to_selected(
                                        placed.clone(),
                                        self.games.players(&m.game_id),
                                    );
                                    if placed.get("first_move_error").unwrap()
                                        == &serde_json::json!(true)
                                    {
                                        let game = self.games.get_game(&m.game_id).unwrap().1;
                                        let filter = doc! {"_id": ObjectId::from_str(&m.game_id.as_str()).unwrap()};
                                        let update = doc! {"$set": bson::to_bson(&game).unwrap()};
                                        let shuuro_games = self.db_shuuro_games.clone();
                                        let b = Box::pin(async move {
                                            let game1 = shuuro_games
                                                .find_one_and_update(filter, update, None);
                                            match game1.await {
                                                _g => {}
                                            };
                                        });
                                        let actor_future = b.into_actor(self);
                                        ctx.spawn(actor_future);
                                        self.games.remove_game(&m.game_id);
                                        res = serde_json::json!({"t": "active_games_count", "cnt": self.games.shuuro_games.len()});
                                        return self.send_message_to_all(res);
                                    }
                                    return ();
                                }
                            }
                        } else if t == "live_game_play" {
                            let m = serde_json::from_str::<GameMove>(&msg.text);
                            if let Ok(m) = m {
                                let played = self.games.play(
                                    &m.game_id,
                                    m.game_move,
                                    &msg.player.username(),
                                );
                                if let Some(mut played) = played {
                                    *played.get_mut("game_id").unwrap() =
                                        serde_json::json!(m.game_id);
                                    let status = &played["status"].as_i64().unwrap();

                                    self.send_message_to_selected(
                                        played,
                                        self.games.players(&m.game_id),
                                    );
                                    if status > &0 {
                                        let game = self.games.get_game(&m.game_id).unwrap().1;
                                        let filter = doc! {"_id": ObjectId::from_str(&m.game_id.as_str()).unwrap()};
                                        let update = doc! {"$set": bson::to_bson(&game).unwrap()};
                                        let shuuro_games = self.db_shuuro_games.clone();
                                        let b = Box::pin(async move {
                                            let game1 = shuuro_games
                                                .find_one_and_update(filter, update, None);
                                            match game1.await {
                                                _g => {}
                                            };
                                        });
                                        let actor_future = b.into_actor(self);
                                        ctx.spawn(actor_future);
                                        self.games.remove_game(&m.game_id);
                                        res = serde_json::json!({"t": "active_games_count", "cnt": self.games.shuuro_games.len()});
                                        return self.send_message_to_all(res);
                                    }
                                    return ();
                                }
                            }
                        } else if t == "live_game_hand" {
                            let m = serde_json::from_str::<GameGetHand>(&msg.text);
                            if let Ok(m) = m {
                                let hand = self.games.get_hand(m.game_id, &msg.player.username());
                                res = serde_json::json!({"t": t, "hand": &hand});
                            }
                        } else if t == "live_game_confirmed" {
                            let m = serde_json::from_str::<GameGetConfirmed>(&msg.text);
                            if let Ok(m) = m {
                                let confirmed = self.games.confirmed_players(&m.game_id);
                                res = serde_json::json!({"t": t, "confirmed": &confirmed});
                            }
                        } else if t == "live_game_draw" {
                            let m = serde_json::from_str::<GameGetConfirmed>(&msg.text);
                            if let Ok(m) = m {
                                let draw = self.games.draw_req(&m.game_id, &msg.player.username());
                                let users = self.games.players(&m.game_id);
                                if draw == 5 {
                                    res = serde_json::json!({"t": t, "draw": true});
                                    self.send_message_to_selected(res, users);
                                    let game = self.games.get_game(&m.game_id).unwrap().1;
                                    let filter = doc! {"_id": ObjectId::from_str(&m.game_id.as_str()).unwrap()};
                                    let update = doc! {"$set": bson::to_bson(&game).unwrap()};
                                    let shuuro_games = self.db_shuuro_games.clone();
                                    let b = Box::pin(async move {
                                        let game1 =
                                            shuuro_games.find_one_and_update(filter, update, None);
                                        match game1.await {
                                            _g => {}
                                        };
                                    });
                                    let actor_future = b.into_actor(self);
                                    ctx.spawn(actor_future);
                                    self.games.remove_game(&m.game_id);
                                    res = serde_json::json!({"t": "active_games_count", "cnt": self.games.shuuro_games.len()});
                                    return self.send_message_to_all(res);
                                } else if draw == -2 {
                                    res = serde_json::json!({"t": t, "draw": false, "player": &msg.player.username()});
                                } else if draw == -3 {
                                    return ();
                                }
                                return self.send_message_to_selected(res, users);
                            }
                        } else if t == "live_game_resign" {
                            let m = serde_json::from_str::<GameGetConfirmed>(&msg.text);
                            if let Ok(m) = m {
                                let resign = self.games.resign(&m.game_id, &msg.player.username());
                                if resign {
                                    let users = self.games.players(&m.game_id);
                                    res = serde_json::json!({"t": t, "resign": true, "player": &msg.player.username()});
                                    self.send_message_to_selected(res, users);
                                    let game = self.games.get_game(&m.game_id).unwrap().1;
                                    let filter = doc! {"_id": ObjectId::from_str(&m.game_id.as_str()).unwrap()};
                                    println!("{}", &game.status);
                                    let update = doc! {"$set": bson::to_bson(&game).unwrap()};
                                    let shuuro_games = self.db_shuuro_games.clone();
                                    let b = Box::pin(async move {
                                        let game1 =
                                            shuuro_games.find_one_and_update(filter, update, None);
                                        match game1.await {
                                            _g => {}
                                        };
                                    });
                                    let actor_future = b.into_actor(self);
                                    ctx.spawn(actor_future);
                                    self.games.remove_game(&m.game_id);
                                    res = serde_json::json!({"t": "active_games_count", "cnt": self.games.shuuro_games.len()});
                                    return self.send_message_to_all(res);
                                }
                                return ();
                            }
                        } else if t == "home_chat_message" {
                            let m = serde_json::from_str::<ChatItem>(&msg.text);
                            if let Ok(mut m) = m {
                                let count = self.chat.iter().fold(0, |mut acc, x| {
                                    if &x.user == &msg.player.username() {
                                        acc += 1;
                                    }
                                    acc
                                });

                                if !&msg.player.reg() {
                                    return ();
                                } else if count == 5 {
                                    return ();
                                }

                                m.update(&msg.player.username());
                                if m.message.len() > 0 && m.message.len() < 50 {
                                    res = m.response();
                                    self.chat.push(m);
                                    return self.send_message_to_all(res);
                                }
                            }
                        } else if t == "home_lobby_full" {
                            res = self.lobby.response()
                        } else if t == "just_stop" {
                            let data_type = &i["game_id"];
                            match data_type {
                                serde_json::Value::String(t) => {
                                    self.games.stop(t.clone());
                                }
                                _ => (),
                            }
                        } else if t == "home_lobby_add" {
                            let m = serde_json::from_str::<LobbyGame>(&msg.text);
                            if let Ok(mut game) = m {
                                if game.is_valid() {
                                    if self.lobby.can_add(&game) {
                                        self.games.can_add(&game.username());
                                        if self.games.can_add(&game.username()) {
                                            res = game.response(&t);
                                            self.lobby.add(game);
                                            return self.send_message_to_all(res);
                                        }
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
                                        let users = game.colors(&msg.player.username());
                                        let mut shuuro_game = ShuuroGame::from(&game);
                                        shuuro_game.white = users[0].clone();
                                        shuuro_game.black = users[1].clone();
                                        let res = game.response(&String::from("home_lobby_remove"));
                                        let deleted = self.lobby.delete(game);
                                        if deleted >= 0 {
                                            self.send_message_to_all(res);
                                        }
                                        let deleted = self.lobby.delete_by_user(&msg.player);
                                        if deleted {
                                            let temp_res = serde_json::json!({"t": "home_lobby_remove_user",
                                                "username": &msg.player.username()});
                                            self.send_message_to_all(temp_res);
                                        }
                                        let db_shuuro_games = self.db_shuuro_games.clone();
                                        self.counter += 1;
                                        let ctx2 = ctx.address().clone();
                                        let b = Box::pin(async move {
                                            let game_added =
                                                db_shuuro_games.insert_one(&shuuro_game, None);
                                            match game_added.await {
                                                g => {
                                                    let id =
                                                        g.ok().unwrap().inserted_id.to_string();
                                                    let game_id = id
                                                        .replace("ObjectId(\"", "")
                                                        .replace("\")", "");
                                                    ctx2.do_send(GameMessage {
                                                        message_type:
                                                            GameMessageType::new_adding_game(
                                                                game_id,
                                                                users,
                                                                shuuro_game,
                                                            ),
                                                    });
                                                }
                                            }
                                        });
                                        let actor_future = b.into_actor(self);
                                        ctx.spawn(actor_future);
                                        return;
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
                serde_json::json!({"t": "active_games_count", "cnt": self.games.shuuro_games.len()}),
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
            serde_json::json!({"t": "active_games_count", "cnt": self.games.shuuro_games.len()});
        let temp_res = serde_json::json!({"t": "home_lobby_remove_user",
                                                "username": &msg.player.username()});
        self.send_message_to_all(temp_res);
        self.send_message_to_all(player_count);
        self.send_message_to_all(matches_count);
    }
}

impl Handler<GameMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: GameMessage, ctx: &mut Context<Self>) {
        match msg.message_type {
            GameMessageType::AddingGame {
                game_id,
                users,
                mut shuuro_game,
            } => {
                shuuro_game.game_id = ObjectId::parse_str(&game_id).unwrap();
                let res = serde_json::json!({"t": "live_game_start", "game_id": game_id, "game_info": &shuuro_game });
                self.games.add_game(game_id.clone(), &shuuro_game);
                self.send_message_to_selected(res, users);
                let res = serde_json::json!({"t": "active_games_count", "cnt": self.games.shuuro_games.len()});
                self.send_message_to_all(res);
            }
            GameMessageType::News {
                news,
                active_player,
            } => {
                let res = serde_json::json!({"t": "home_news", "news": news });
                self.send_message(&active_player, res);
            }
        }
    }
}
