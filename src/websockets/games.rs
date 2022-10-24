use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bson::DateTime as DT;
use chrono::Utc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shuuro::{init, position::Outcome, Color, Move, PieceType};

use crate::{
    arc2,
    database::{mongo::ShuuroGame, queries::update_entire_game, redis::UserSession},
};

use super::{time_control::TimeCheck, GameGet, LiveGameMove, MessageHandler, MsgDatabase};

pub struct ShuuroGames {
    all: Arc<Mutex<HashMap<String, ShuuroGame>>>,
    unfinished: Arc<Mutex<Vec<String>>>
}

impl Default for ShuuroGames {
    fn default() -> Self {
        init();
        Self {
            all: arc2(HashMap::new()),
            unfinished: arc2(vec![])
        }
    }
}

impl ShuuroGames {
    /// Add new game to live games.
    pub fn add_game(&self, game: ShuuroGame) -> usize {
        let mut all = self.all.lock().unwrap();
        all.insert(String::from(&game._id), game);
        all.len()
    }

    /// Remove game after end.
    pub async fn remove_game(&self, db: &Collection<ShuuroGame>, id: &String) {
        let mut all = self.all.lock().unwrap();
        if let Some(game) = all.remove(id) {
            let db = db.clone();
            tokio::spawn(async move {
                update_entire_game(&db, &game).await;
            });
        }
    }

    /// Count all games.
    pub fn game_count(&self) -> usize {
        self.all.lock().unwrap().len()
    }

    /// Load games from db
    pub fn load_unfinished(&self, hm: HashMap<String, ShuuroGame>) {
        let mut temp = HashMap::new();
        let mut v = vec![];
        for mut i in hm {
            //self.ws.players.new_spectators(&i.0);
            v.push(String::from(&i.0));
            if i.1.current_stage == 0 {
                let hands = format!("{}{}", &i.1.hands[0], &i.1.hands[1]);
                i.1.shuuro.0.set_hand(hands.as_str());
                temp.insert(i.0, i.1);
            } else if i.1.current_stage == 1 {
                i.1.shuuro.1.set_sfen_history(i.1.history.1.clone());
                let _ = i.1.shuuro.1.set_sfen(&i.1.sfen);
                temp.insert(i.0, i.1);
            } else if i.1.current_stage == 2 {
                i.1.shuuro.2.set_sfen_history(i.1.history.2.clone());
                let _ = i.1.shuuro.2.set_sfen(&i.1.sfen);
                temp.insert(i.0, i.1);
            }
        }
        *self.all.lock().unwrap() = temp;
        *self.unfinished.lock().unwrap() = v;
    }

    pub fn get_unfinished(&self) -> Vec<String> {
        let unfinished = self.unfinished.lock().unwrap();
        let games = unfinished.clone();
        drop(unfinished);
        games
    }

    pub fn del_unfinished(&self) {
        let mut unfinished = self.unfinished.lock().unwrap();
        *unfinished = vec![];
        drop(unfinished);
    }

    // SHOP PART

    pub fn change_variant(&self, id: &String, variant: &String) {
        let mut all = self.all.lock().unwrap();
        if let Some(g) = all.get_mut(id) {
            if variant == "shuuro12fairy" {
                g.shuuro.0.change_variant();
                g.shuuro.1.update_variant();
                g.shuuro.2.update_variant();
                drop(all); 
            }
        }
    }

    /// Get hand for active player.
    pub fn get_hand(&self, id: &String, user: &UserSession) -> Option<String> {
        let all = self.all.lock().unwrap();
        if let Some(g) = all.get(id) {
            if let Some(index) = self.player_index(&g.players, &user.username) {
                let color = Color::from(index);
                return Some(g.shuuro.0.to_sfen(color));
            }
        }
        None
    }

    /// Get confirmed players.
    pub fn get_confirmed(&self, id: &String) -> Option<[bool; 2]> {
        let all = self.all.lock().unwrap();
        if let Some(g) = all.get(id) {
            let confirmed = self.confirmed(g);
            return Some(confirmed);
        }
        None
    }

    fn confirmed(&self, game: &ShuuroGame) -> [bool; 2] {
        let mut confirmed = [false, false];
        confirmed[0] = game.shuuro.0.is_confirmed(shuuro::Color::White);
        confirmed[1] = game.shuuro.0.is_confirmed(shuuro::Color::Black);
        confirmed
    }

    /// Buy one piece.
    pub fn buy(&self, json: &GameGet, player: &String) -> Option<LiveGameMove> {
        let mut all = self.all.lock().unwrap();
        if let Some(game) = all.get_mut(&json.game_id) {
            if let Some(p) = self.player_index(&game.players, player) {
                if let Some(_c) = game.tc.click(p) {
                    return self.buy_piece(json, game, p);
                } else {
                    return Some(LiveGameMove::LostOnTime(p));
                }
            }
        }
        None
    }

    /// Buy piece.
    fn buy_piece(&self, json: &GameGet, game: &mut ShuuroGame, p: usize) -> Option<LiveGameMove> {
        if let Some(m) = Move::from_sfen(&json.game_move) {
            match m {
                Move::Buy { piece } => {
                    let player_color = Color::from(p);
                    if player_color == piece.color {
                        if let Some(confirmed) = game.shuuro.0.play(m) {
                            game.draws = [false, false];
                            game.hands[p] = game.shuuro.0.to_sfen(player_color);
                            if confirmed[player_color as usize] == true {
                                return Some(LiveGameMove::BuyMove(confirmed));
                            }
                        }
                    } else {
                    }
                }
                _ => (),
            }
        } else {
            // If move is wrong then confirm player choice.
            game.shuuro.0.confirm(Color::from(p));
            return Some(LiveGameMove::BuyMove(self.confirmed(&game)));
        }
        None
    }

    /// Transfer hand from shop to deploy part.
    pub fn load_shop_hand(&self, game: &mut ShuuroGame) -> String {
        let w = game.shuuro.0.to_sfen(Color::White);
        let b = game.shuuro.0.to_sfen(Color::Black);
        let hand = format!("{}{}", &w, &b);
        game.hands = [w, b];
        let sfen = "57/57/57/57/57/57/57/57/57/57/57/57 w";
        game.shuuro.1.set_hand(hand.as_str());
        game.shuuro.1.set_sfen(&sfen).ok();
        hand
    }

    /// Check if deployment is over by checking player's hands.
    pub fn is_deployment_over(&self, g: &ShuuroGame) -> bool {
        let mut completed: [bool; 3] = [false, false, false];
        let color_iter = Color::iter();
        for i in color_iter {
            completed[i.index()] = g.shuuro.1.is_hand_empty(&i, PieceType::Plinth);
        }
        completed[2] = true;
        !completed.contains(&false)
    }

    // DEPLOY PART

    /// Place piece on board.
    pub fn place_move(&self, json: &GameGet, player: &String) -> Option<LiveGameMove> {
        let mut all = self.all.lock().unwrap();
        if let Some(game) = all.get_mut(&json.game_id) {
            if game.current_stage != 1 {
                return None;
            }
            if let Some(index) = self.player_index(&game.players, player) {
                if game.shuuro.1.side_to_move() != Color::from(index) {
                    return None;
                }
                if let Some(clocks) = game.tc.click(index) {
                    game.clocks = game.tc.clocks;
                    game.last_clock = DT::now();
                    return self.place_piece(json, index, clocks, game);
                } else {
                }
            }
        }
        None
    }

    /// Get hands for both player.
    fn get_hands(&self, game: &ShuuroGame) -> [String; 2] {
        [
            game.shuuro.1.get_hand(Color::White),
            game.shuuro.1.get_hand(Color::Black),
        ]
    }

    /// Placing piece on board. Returns LiveGameMove.
    fn place_piece(
        &self,
        json: &GameGet,
        index: usize,
        clocks: [u64; 2],
        game: &mut ShuuroGame,
    ) -> Option<LiveGameMove> {
        if let Some(gm) = Move::from_sfen(&json.game_move.as_str()) {
            match gm {
                Move::Put { to, piece } => {
                    if Color::from(index) == piece.color {
                        if let Some(s) = game.shuuro.1.place(piece, to) {
                            game.draws = [false, false];
                            let mut fme = false;
                            let m = s.split("_").next().unwrap().to_string();
                            let tf = self.is_deployment_over(&game);
                            if tf {
                                fme = self.set_fight(game);
                            }
                            game.side_to_move =
                                self.other_index(game.shuuro.1.side_to_move()) as u8;
                            game.sfen = game.shuuro.1.generate_sfen();
                            game.hands = self.get_hands(&game);
                            game.history.1.push((s, 1));
                            return Some(LiveGameMove::PlaceMove(
                                m,
                                clocks,
                                fme,
                                tf,
                                game.players.clone(),
                            ));
                        }
                    } else {
                    }
                }
                _ => (),
            }
        }
        None
    }

    /// Transfer data from deployment to fighting part of game.
    pub fn set_fight(&self, game: &mut ShuuroGame) -> bool {
        game.current_stage = 2;
        game.tc.update_stage(2);
        game.last_clock = DT::now();
        let sfen = game.shuuro.1.generate_sfen();
        let outcome = game.shuuro.2.set_sfen(&sfen.as_str());
        if let Ok(_o) = outcome {
            self.update_status(game);
        }
        game.shuuro.2.in_check(game.shuuro.2.side_to_move().flip())
    }

    /// Make a move.
    pub fn fight_move(&self, json: &GameGet, player: &String) -> Option<LiveGameMove> {
        let mut all = self.all.lock().unwrap();
        if let Some(game) = all.get_mut(&json.game_id) {
            if game.current_stage != 2 {
                return None;
            }
            if let Some(index) = self.player_index(&game.players, player) {
                if game.shuuro.2.side_to_move() != Color::from(index) {
                    return None;
                }
                if let Some(clocks) = game.tc.click(index) {
                    game.draws = [false, false];
                    game.clocks = game.tc.clocks;
                    game.last_clock = DT::now();
                    return self.make_move(json, game, index, clocks);
                } else {
                }
            }
        }
        None
    }

    /// Returns LiveGameMove if move is legal.
    pub fn make_move(
        &self,
        json: &GameGet,
        game: &mut ShuuroGame,
        index: usize,
        clocks: [u64; 2],
    ) -> Option<LiveGameMove> {
        if let Some(gm) = Move::from_sfen(&json.game_move.as_str()) {
            match gm {
                Move::Normal {
                    from,
                    to,
                    promote: _,
                } => {
                    if let Some(piece) = game.shuuro.2.piece_at(from) {
                        if Color::from(index) == piece.color {
                            if let Ok(_) = game
                                .shuuro
                                .2
                                .play(from.to_string().as_str(), to.to_string().as_str())
                            {
                                let outcome = self.update_status(game);
                                let stm = self.other_index(game.shuuro.2.side_to_move());
                                game.side_to_move = stm as u8;
                                game.sfen = game.shuuro.2.generate_sfen();
                                let m = game.shuuro.2.get_sfen_history().last().unwrap();
                                game.history.2.push(m.clone());
                                return Some(LiveGameMove::FightMove(
                                    String::from(&json.game_move),
                                    clocks,
                                    game.status,
                                    String::from(&game.result),
                                    game.players.clone(),
                                    outcome,
                                ));
                            }
                        }
                    } else {
                    }
                }
                _ => (),
            }
        }
        None
    }

    /// After match is finished, update status.
    pub fn update_status(&self, game: &mut ShuuroGame) -> String {
        let outcome = game.shuuro.2.outcome();
        match outcome {
            Outcome::Check { color: _ } => {
                game.status = -1;
            }
            Outcome::Nothing => {
                game.status = -1;
            }
            Outcome::Stalemate => {
                game.status = 3;
            }
            Outcome::Draw => {
                game.status = 5;
            }
            Outcome::DrawByRepetition => game.status = 4,
            Outcome::DrawByMaterial => game.status = 6,
            Outcome::Checkmate { color } => {
                game.status = 1;
                game.result = color.to_string();
            }
            Outcome::MoveOk => {
                game.status = -1;
            }
            Outcome::MoveNotOk => {
                game.status = -2;
            }
        }
        outcome.to_string()
    }

    pub fn set_deploy(&self, id: &String) -> Option<Value> {
        if let Some(game) = self.all.lock().unwrap().get_mut(id) {
            game.current_stage = 1;
            let hand = self.load_shop_hand(game);
            game.shuuro.1.generate_plinths();
            game.sfen = game.shuuro.1.to_sfen();
            game.side_to_move = 0;
            let value = serde_json::json!({
                "t": "redirect_deploy",
                "path": format!("/shuuro/1/{}", id),
                "hand": hand,
                "last_clock": Utc::now(),
                "side_to_move": "w",
                "w": String::from(&game.players[0]),
                "b": String::from(&game.players[1]),
                "sfen": game.sfen
            });
            return Some(value);
        }
        None
    }

    /// DRAW PART

    pub fn draw_req(&self, id: &String, username: &String) -> Option<(i8, [String; 2])> {
        if let Some(game) = self.all.lock().unwrap().get_mut(id) {
            if let Some(index) = self.player_index(&game.players, username) {
                game.draws[index] = true;
                if !game.draws.contains(&false) {
                    game.status = 5;
                    return Some((5, game.players.clone()));
                } else {
                    return Some((-2, game.players.clone()));
                }
            }
        }
        None
    }

    // HELPER METHODS

    pub fn get_players(&self, id: &String) -> Option<[String; 2]> {
        if let Some(game) = self.all.lock().unwrap().get(id) {
            return Some(game.players.clone());
        }
        None
    }

    /// Returns index of player if it exist.
    fn player_index(&self, p: &[String; 2], u: &String) -> Option<usize> {
        p.iter().position(|x| x == u)
    }

    /// Opposite index of specified.
    fn other_index(&self, color: Color) -> usize {
        let b: bool = color as usize != 0;
        usize::from(!b)
    }

    /// CLOCK PART

    /// After every 500ms, this function returns who lost on time.
    pub fn clock_status(
        &self,
        time_check: &Arc<Mutex<TimeCheck>>,
    ) -> Option<(Value, Value, [String; 2])> {
        let time_check = time_check.lock().unwrap();
        if let Some(g) = self
            .all
            .lock()
            .unwrap()
            .get_mut(&String::from(&time_check.id))
        {
            if time_check.both_lost {
                g.status = 5;
            } else {
                g.status = 8;
                g.result = {
                    if time_check.lost == 0 {
                        String::from("w")
                    } else if time_check.lost == 1 {
                        String::from("b")
                    } else {
                        String::from("")
                    }
                };
            }
            let res = serde_json::json!({
            "t": "live_game_lot",
            "game_id": &g._id,
            "status": g.status,
            "result": String::from(&g.result)});
            let tv_res = serde_json::json!({"t": "live_game_end", "game_id": String::from(&g._id)});
            let tv_res = serde_json::json!({"t": "tv_game_update", "g": tv_res});
            return Some((res, tv_res, g.players.clone()));
        }
        drop(time_check);
        None
    }

    /// Check clocks for current stage.
    pub fn check_clocks(&self, time_check: &Arc<Mutex<TimeCheck>>) {
        let id = String::from(&time_check.lock().unwrap().id);
        if let Some(game) = self.all.lock().unwrap().get(&id) {
            drop(id);
            if game.current_stage == 0 {
                let durations = [game.tc.current_duration(0), game.tc.current_duration(1)];
                let confirmed = self.confirmed(game);
                if durations == [None, None] {
                    if &confirmed == &[false, false] {
                        time_check.lock().unwrap().both_lost();
                    } else if let Some(confirmed) = confirmed.iter().position(|i| i == &false) {
                        time_check.lock().unwrap().lost(confirmed);
                    }
                } else if let Some(index) = durations.iter().position(|p| p == &None) {
                    time_check.lock().unwrap().lost(index);
                }
            } else if game.current_stage != 0 {
                let stm = game.side_to_move;
                if let None = game.tc.current_duration(stm as usize) {
                    time_check.lock().unwrap().lost(stm as usize);
                }
            }
            return;
        }
        time_check.lock().unwrap().dont_exist();
    }

    /// Get live or archived game(if it exist).
    pub async fn get_game<'a>(
        &self,
        id: &String,
        _db: &Collection<ShuuroGame>,
        s: &'a MessageHandler<'a>,
    ) -> Option<ShuuroGame> {
        let id = String::from(id);
        let all = self.all.lock().unwrap();
        if let Some(g) = all.get(&id) {
            return Some(g.clone());
        }
        let _ = s.db_tx.clone().send(MsgDatabase::GetGame(String::from(id)));

        return None;
    }

    /// Live sfen used for TV.
    pub fn live_sfen(&self, id: &String) -> Option<(u8, String)> {
        if let Some(g) = self.all.lock().unwrap().get(id) {
            return Some((g.current_stage, String::from(&g.sfen)));
        }
        None
    }

    /// Resign if this player exist in game.
    pub fn resign(&self, id: &String, username: &String) -> Option<[String; 2]> {
        if let Some(g) = self.all.lock().unwrap().get_mut(id) {
            if let Some(index) = self.player_index(&g.players, &username) {
                g.status = 7;
                g.result = Color::from(index).to_string();
                g.tc.click(index);
                g.last_clock = DT::now();
                return Some(g.players.clone());
            }
        }
        None
    }

    /// Get 20 matches for tv.
    pub fn get_tv(&self) -> Vec<TvGame> {
        let c = 0;
        let mut games = vec![];
        let all = self.all.lock().unwrap();
        for i in all.iter() {
            if c == 20 {
                break;
            }
            let f = &i.1.sfen;
            if f == "" {
                continue;
            }
            let id = &i.1._id;
            let w = &i.1.players[0];
            let b = &i.1.players[1];
            let t = "live_tv";
            let tv = TvGame::new(t, id, w, b, f);
            games.push(tv);
        }
        games
    }

    /// Before closing server save on exit.
    pub async fn save_on_exit(&self, db: &Collection<ShuuroGame>) {
        let all = self.all.lock().unwrap().clone();
        for (_, game) in all {
            update_entire_game(db, &game).await;
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TvGame {
    pub t: String,
    pub game_id: String,
    pub w: String,
    pub b: String,
    pub sfen: String,
}

impl TvGame {
    pub fn new(t: &str, game_id: &str, w: &str, b: &str, fen: &str) -> Self {
        Self {
            t: String::from(t),
            game_id: String::from(game_id),
            w: String::from(w),
            b: String::from(b),
            sfen: String::from(fen),
        }
    }
}

fn _other_index(color: usize) -> usize {
    let b: bool = color != 0;
    usize::from(!b)
}
