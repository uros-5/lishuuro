pub mod database;
pub mod lichess;
pub mod routes;
pub mod websockets;

use std::env;
use std::sync::{Arc, Mutex};

use axum::{http::HeaderValue, routing::get, Router};
use database::Database;
use minijinja::Environment;
use routes::{
    callback, game_axum, game_vue, games_axum, games_vue, home, how_to_play, logged,
    login, save_state, tv, vue_user,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use websockets::{channels::WsState, handler::websocket_handler};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let db = Database::new().await;
    let db = Arc::new(db);
    let ws = WsState::new(db.clone()).await;
    let ws = Arc::new(ws);
    ws.send_ws(ws.clone()).await;
    let state = AppState::new(db, ws);
    let cors = cors(state.db.key.prod);
    let app = Router::new()
        .route("/login", get(login))
        .route("/", get(home))
        .route("/game/{id}", get(game_axum))
        .route("/how-to-play-shuuro", get(how_to_play))
        .route("/tv", get(tv))
        .route("/@/{username}", get(games_axum))
        .route("/callback", get(callback))
        .route("/logged", get(logged))
        .route("/vue_user", get(vue_user))
        .route("/vue/game/{id}", get(game_vue))
        .route("/vue/@/{username}/{page}", get(games_vue))
        .route("/ws/", get(websocket_handler))
        .route("/shutdown", get(save_state))
        .with_state(state)
        .nest_service("/assets", ServeDir::new("./assets/assets"))
        .nest_service("/board", ServeDir::new("./assets/board"))
        .nest_service("/fonts", ServeDir::new("./assets/fonts"))
        .nest_service("/images", ServeDir::new("./assets/images"))
        .nest_service("/pieces", ServeDir::new("./assets/pieces"))
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn cors(prod: bool) -> CorsLayer {
    let addr = prod_url(prod);
    let cors = CorsLayer::new();
    cors.allow_origin(addr.1.parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
}

pub fn prod_url(prod: bool) -> (&'static str, &'static str) {
    if prod {
        return ("https://lishuuro.org", "https://lishuuro.org");
    }

    let vue = env::var("VUE")
        .unwrap_or(String::from("true"))
        .parse::<bool>()
        .unwrap();
    if vue {
        return ("http://localhost:3000", "http://localhost:5173");
    }
    ("http://192.168.1.17:3000", "http://192.168.1.17:3000")
}

pub fn arc2<T>(data: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(data))
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub ws: Arc<WsState>,
    pub jinja: Arc<Environment<'static>>,
}

impl AppState {
    pub fn new(db: Arc<Database>, ws: Arc<WsState>) -> Self {
        let mut jinja = Environment::new();
        jinja
            .add_template_owned(
                "index.j2",
                std::fs::read_to_string("./assets/index.j2").unwrap(),
            )
            .unwrap();

        let jinja = Arc::new(jinja);
        Self { db, ws, jinja }
    }
}
