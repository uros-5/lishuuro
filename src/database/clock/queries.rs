use std::collections::HashMap;

use base64::{prelude::BASE64_STANDARD, Engine};
use bson::doc;
use futures::TryStreamExt;
use mongodb::{options::FindOptions, Collection};
use rand::Rng;

use crate::{
    database::{
        model::{Player, ShuuroGame},
        redis::UserSession,
    },
    lichess::login_helpers::base64_encode,
};

pub fn random_username() -> String {
    let username = BASE64_STANDARD.encode(rand::rng().random::<[u8; 6]>());
    format!("Anon-{}", username.replace(['+', '/', '='], ""))
}

pub fn random_game_id() -> String {
    let random_bytes = rand::rng().random::<[u8; 9]>();

    format!(
        "{}",
        base64_encode(random_bytes).replace(['+', '/', '=', '-', '_'], "")
    )
}

/// Create new player.
pub async fn create_player(db: &Collection<Player>) -> String {
    loop {
        let username = random_username();
        let player = Player {
            _id: String::from(&username),
            reg: false,
            created_at: bson::DateTime::now(),
        };
        let res = db.insert_one(&player).await;
        // Player is added, therefore it's new.
        if res.is_ok() {
            return (username).to_string();
        }
    }
}

pub async fn get_player(db: &Collection<Player>, username: &str) -> Option<Player> {
    let player = db
        .find_one(doc! {"_id": String::from(username)})
        .await
        .ok()?;
    return player;
}

/// Check if player(with lichess account) exist
pub async fn player_exist(
    db: &Collection<Player>,
    username: &str,
    session: &UserSession,
) -> Option<UserSession> {
    let player = db
        .find_one(doc! {"_id": String::from(username)})
        .await
        .ok()?;

    let mut session = session.clone();
    session.is_new = true;
    session.username = username.to_string();
    session.reg = true;

    match player {
        Some(_) => Some(session),
        None => {
            let player = Player::from(&session);
            let _player = db.insert_one(player).await;
            Some(session)
        }
    }
}

pub async fn game_id(db: &Collection<ShuuroGame>) -> String {
    loop {
        let id = random_game_id();
        if get_game_db(db, &id).await.is_none() {
            return id;
        }
    }
}

pub async fn get_game_db(
    db: &Collection<ShuuroGame>,
    id: &String,
) -> Option<ShuuroGame> {
    let id = String::from(id);
    let filter = doc! {"_id": id};
    if let Ok(r) = db.find_one(filter).await {
        if let Some(g) = r {
            return Some(g);
        }
    }
    None
}

pub async fn add_game_to_db(
    db: &Collection<ShuuroGame>,
    game: ShuuroGame,
    started: bool,
) -> ShuuroGame {
    if started {
        return game;
    }
    if let Err(_res) = db.insert_one(&game).await {}
    game
    // live_game_start(game)
}

pub async fn remove_game(db: &Collection<ShuuroGame>, id: String) -> String {
    let query = doc! {"_id": &id};
    if let Err(_res) = db.delete_one(query).await {}
    id
}

pub async fn update_entire_game(db: &Collection<ShuuroGame>, game: &ShuuroGame) {
    let query = doc! {"_id": &game._id};
    // let mut game = game.clone();
    // game.tc.play(color)
    let update = doc! {"$set": bson::to_bson(&game).unwrap()};
    db.update_one(query, update).await.ok();
}

pub async fn get_player_games(
    db: &Collection<ShuuroGame>,
    username: &String,
    page: u64,
) -> Option<Vec<ShuuroGame>> {
    let options = FindOptions::builder()
        // .projection(doc! {"history": 0, "credits": 0, "hands": 0})
        .sort(doc! {"last_clock": -1})
        .skip(Some(page * 5))
        .limit(Some(5))
        .build();
    let filter = doc! {"players": {"$in": [username] }, "status":{"$gt": 0} };
    let q = db
        .clone_with_type::<ShuuroGame>()
        .find(filter)
        .with_options(options)
        .await;
    if let Ok(res) = q {
        let mut games: Vec<ShuuroGame> =
            res.try_collect().await.unwrap_or_else(|_| vec![]);
        games.iter_mut().for_each(|game| {
            let moves_count = game.history.2.len();
            game.history = (vec![format!("{}", moves_count)], vec![], vec![]);
        });
        return Some(games);
    }
    None
}

pub async fn unfinished(db: &Collection<ShuuroGame>) -> HashMap<String, ShuuroGame> {
    let filter = doc! {"status" : {"$lt": 0}};
    let mut hm = HashMap::new();
    let c = db.find(filter);
    if let Ok(c) = c.await {
        let games: Vec<ShuuroGame> =
            c.try_collect().await.unwrap_or_else(|_| vec![]);
        for g in games {
            if g.players.contains(&String::from("")) {
                remove_game(db, g._id).await;
                continue;
            }
            hm.insert(String::from(&g._id), g);
        }
    }
    hm
}
