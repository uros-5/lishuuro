use crate::websockets::channels::game_requests::GameRequest;
use typeshare::typeshare;

use super::{clock::time_control::TimeControl, serde_helpers::*};
use bson::DateTime;
use chrono::Duration;
use mongodb::{options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use shuuro::{SubVariant, Variant};
use std::env;

#[derive(Clone)]
pub struct Mongo {
    pub players: Collection<Player>,
    pub games: Collection<ShuuroGame>,
}

impl Mongo {
    /// Create mongodb connection for all collections.
    pub async fn new() -> Self {
        let addr = env::var("MONGO").expect("mongo addr not found");
        let mut client_options = ClientOptions::parse(addr)
            .await
            .expect("No client available");
        client_options.app_name = Some("lishuuro".to_string());
        let client = Client::with_options(client_options).expect("client not found");
        let db = client.database("lishuuro");
        let players = db.collection::<Player>("users");
        let games = db.collection::<ShuuroGame>("shuuroGames");
        Mongo { players, games }
    }
}

pub type History = (Vec<String>, Vec<String>, Vec<String>);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[typeshare]
/// Representing one player
pub struct Player {
    pub _id: String,
    pub reg: bool,
    #[typeshare(serialized_as = "Value")]
    pub created_at: DateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[typeshare]
pub struct ShuuroGame {
    pub _id: String,
    #[serde(serialize_with = "duration_to_u64")]
    #[serde(deserialize_with = "str_to_duration")]
    #[typeshare(serialized_as = "u8")]
    pub min: Duration,
    #[serde(serialize_with = "duration_to_u64")]
    #[serde(deserialize_with = "str_to_duration")]
    #[typeshare(serialized_as = "u8")]
    pub incr: Duration,
    pub players: [String; 2],
    pub side_to_move: u8,
    #[serde(serialize_with = "duration_to_array")]
    #[serde(deserialize_with = "array_to_duration")]
    #[typeshare(serialized_as = "[u8; 2]")]
    pub clocks: [Duration; 2],
    #[typeshare(serialized_as = "String")]
    pub last_clock: DateTime,
    pub current_stage: u8,
    pub result: u8,
    pub status: i32,
    #[serde(serialize_with = "serialize_variant")]
    #[serde(deserialize_with = "deserialize_variant")]
    #[typeshare(serialized_as = "u8")]
    pub variant: Variant,
    pub credits: [u16; 2],
    pub hands: [String; 2],
    pub sfen: String,
    #[typeshare(serialized_as = "[Vec<String>; 3]")]
    pub history: History,
    pub game_start: String,
    pub placement_start: String,
    pub tc: TimeControl,
    #[serde(skip)]
    pub draws: [bool; 2],
    #[serde(serialize_with = "serialize_subvariant")]
    #[serde(deserialize_with = "deserialize_subvariant")]
    #[typeshare(serialized_as = "Option<u8>")]
    pub sub_variant: Option<SubVariant>,
}

impl From<(&GameRequest, &[String; 2], &str)> for ShuuroGame {
    fn from(f: (&GameRequest, &[String; 2], &str)) -> Self {
        let clock = Duration::seconds(60 * f.0.minutes + f.0.incr);
        Self {
            _id: String::from(f.2),
            min: Duration::seconds(f.0.minutes * 60),
            incr: Duration::seconds(f.0.incr),
            players: f.1.clone(),
            side_to_move: 0,
            clocks: [clock, clock],
            last_clock: DateTime::now(),
            current_stage: 0,
            result: 2,
            status: -2,
            variant: f.0.variant,
            credits: [800, 800],
            hands: [String::from(""), String::from("")],
            sfen: String::from(""),
            history: (vec![], vec![], vec![]),
            game_start: String::default(),
            placement_start: String::default(),
            tc: TimeControl::new(f.0.minutes, f.0.incr),
            draws: [false, false],
            sub_variant: f.0.sub_variant,
        }
    }
}
