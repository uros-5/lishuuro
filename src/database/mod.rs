use std::{env, sync::Arc};

use model::Mongo;
use redis::RedisCli;

use crate::{lichess::MyKey, websockets::channels::ai::Pockets};

pub mod clock;
pub mod model;
pub mod redis;
pub mod serde_helpers;

#[derive(Clone)]
pub struct Database {
    pub redis: RedisCli,
    pub mongo: Mongo,
    pub key: MyKey,
    pub mod1: String,
    pub pockets: Arc<Pockets>,
}

impl Database {
    /// Create databases.
    pub async fn new() -> Self {
        let redis = RedisCli::default().await;
        let mongo = Mongo::new().await;
        let key = MyKey::default();
        let mod1 = env::var("LOGIN_STATE").unwrap();
        let pockets = Arc::new(Pockets::new());
        Self {
            redis,
            mongo,
            key,
            mod1,
            pockets,
        }
    }
}
