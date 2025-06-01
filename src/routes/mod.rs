use std::collections::HashMap;
use typeshare::typeshare;

use axum::{
    extract::{Path, Query, State},
    response::{Html, Redirect},
    Json,
};
use hyper::{HeaderMap, StatusCode};
use minijinja::context;
use serde::Serialize;
use shuuro::Color;
use tokio::sync::oneshot;

use crate::{
    database::{
        clock::queries::{get_game_db, get_player, get_player_games, player_exist},
        model::{Player, ShuuroGame},
        redis::{UserSession, VueUser},
    },
    lichess::login::{get_lichess_token, get_lichess_user, login_url, LichessError},
    prod_url,
    websockets::channels::games::GamesMessage,
    AppState,
};

pub async fn home(
    mut _user: UserSession,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = state.jinja.get_template("index.j2").unwrap();
    let ctx = context!( description => "Play shuuro", title => "Home - lishuuro.org", props => "{}");
    let output = template.render(ctx).unwrap();
    Ok(Html(output))
}

pub async fn tv(
    mut _user: UserSession,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = state.jinja.get_template("index.j2").unwrap();
    let ctx = context!( description => "Watch TV", title => "Watch TV - lishuuro.org", props => "{}");
    let output = template.render(ctx).unwrap();
    Ok(Html(output))
}

pub async fn how_to_play(
    mut _user: UserSession,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = state.jinja.get_template("index.j2").unwrap();
    let ctx = context!( description => "How to play shuuro?", title => "How to play shuuro?- lishuuro.org", props => "{}");
    let output = template.render(ctx).unwrap();
    Ok(Html(output))
}

pub async fn login(
    mut user: UserSession,
    State(state): State<AppState>,
) -> Redirect {
    let key = &state.db.key;
    let mut redis = state.db.redis.clone();
    let url = login_url(&key.login_state, key.prod);
    user.code_verifier = url.1;
    redis.set_session(&user.session, user.clone(), true).await;
    Redirect::permanent(url.0.as_str())
}

pub async fn callback(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    user: UserSession,
) -> Result<Redirect, LichessError> {
    let key = &state.db.key;
    let mongo = &state.db.mongo;
    let mut redis = state.db.redis.clone();
    let r = prod_url(key.prod);
    let r = format!("{}/logged", r.1);
    let Some(code) = params.get(&String::from("code")) else {
        return Ok(Redirect::permanent(r.as_str()));
    };
    let lichess_token =
        get_lichess_token(code, &user.code_verifier, key.prod).await?;

    let lichess_user = get_lichess_user(lichess_token.access_token).await?;
    let player = player_exist(&mongo.players, &lichess_user, &user).await;
    if let Some(player) = player {
        let session = String::from(&player.session);
        redis.set_session(&session, player, true).await;
    }

    Ok(Redirect::permanent(r.as_str()))
}

pub async fn logged(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = state.jinja.get_template("index.j2").unwrap();
    let description = "Lichess account verified";
    let title = format!("Player {} verified - lishuuro.org", &username);
    let ctx = context!( description => description, title => title, props => "{}");
    let output = template.render(ctx).unwrap();
    Ok(Html(output))
}
pub async fn vue_user(user: UserSession) -> (HeaderMap, Json<VueUser>) {
    let headers = user.headers();
    (headers, Json(VueUser::from(&user)))
}

pub async fn games_axum(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = state.jinja.get_template("index.j2").unwrap();
    let description = format!("Check profile for {}", &username);
    let title = format!("{} profile - lishuuro.org", &username);
    let ctx = context!( description => description, title => title, props => "{}");
    let output = template.render(ctx).unwrap();
    Ok(Html(output))
}

pub async fn games_vue(
    Path((username, page)): Path<(String, u64)>,
    State(state): State<AppState>,
) -> Json<UserProfileGames> {
    Json(get_games(username, page, state).await)
}

pub async fn get_games(
    username: String,
    page: u64,
    state: AppState,
) -> UserProfileGames {
    let mut player = None;
    if page < 2 {
        player = get_player(&state.db.mongo.players, &username).await;
    }
    let games = get_player_games(&state.db.mongo.games, &username, page).await;
    UserProfileGames { player, games }
}

async fn get_game(game: String, state: AppState) -> Option<ShuuroGame> {
    let (tx, rx) = oneshot::channel();
    let _ = state
        .ws
        .games
        .send(GamesMessage::GetGame(tx, game.to_string()))
        .await;

    match rx.await {
        Ok(game) => {
            return Some(game);
        }
        Err(_) => {
            let game = get_game_db(&state.db.mongo.games, &game).await;
            if let Some(game) = game {
                return Some(game);
            }
        }
    }
    None
}

pub async fn game_vue(
    Path(path): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ShuuroGame>, StatusCode> {
    let game = get_game(path, state).await;
    match game {
        Some(game) => Ok(Json(game)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn game_axum(
    mut _user: UserSession,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = state.jinja.get_template("index.j2").unwrap();
    let Some(game) = get_game(id, state.clone()).await else {
        return Err(StatusCode::NOT_FOUND);
    };
    let message = {
        match game.players.iter().position(|player| player == "") {
            Some(index) => {
                let name = &game.players[Color::from(index).flip() as usize];
                format!("{} is waiting for you.. - lishuuro.org", name)
            }
            None => format!(
                "{} vs {} - lishuuro.org",
                &game.players[0], &game.players[1]
            ),
        }
    };
    let ctx = context!( description => &message, title => &message, props => game);
    let output = template.render(ctx).unwrap();
    Ok(Html(output))
}

pub async fn save_state(user: UserSession, State(state): State<AppState>) {
    if user.username == "iiiurosiii" {
        let _ = state.ws.games.send(GamesMessage::SaveState).await;
    }
}

#[derive(Serialize)]
#[typeshare]
pub struct UserProfileGames {
    player: Option<Player>,
    games: Option<Vec<ShuuroGame>>,
}
