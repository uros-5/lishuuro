use async_session::Session;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    http::HeaderValue,
    RequestPartsExt,
};
use axum_extra::{headers::Cookie, typed_header::TypedHeader};
use bson::DateTime;
use hyper::{header::SET_COOKIE, HeaderMap, StatusCode};
use mongodb::Collection;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::env;

use crate::{lichess::cookies, AppState};

use super::{clock::queries::create_player, model::Player};

pub const AXUM_SESSION_COOKIE_NAME: &str = "axum_session";

/// Struct representing current user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub username: String,
    pub reg: bool,
    pub code_verifier: String,
    pub session: String,
    pub is_new: bool,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub cookie_value: CookieValue,
}

impl UserSession {
    pub fn new(
        username: &str,
        session: &str,
        reg: bool,
        code_verifier: &str,
        cookie_value: CookieValue,
    ) -> Self {
        Self {
            username: String::from(username),
            reg,
            code_verifier: String::from(code_verifier),
            session: String::from(session),
            is_new: true,
            cookie_value,
        }
    }

    pub fn not_new(&mut self) {
        self.is_new = false;
    }

    pub fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if self.is_new {
            let max_age = 60 * 60 * 24 * 365;
            let cookie = format!(
                "{}={}; {} max-age={}; Path=/",
                AXUM_SESSION_COOKIE_NAME,
                &self.session,
                &self.cookie_value.response(),
                max_age
            );
            headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
        }
        headers
    }
}

#[derive(Clone, Debug)]
pub struct CookieValue {
    pub same_site: String,
    pub secure: String,
    pub http_only: String,
}

impl CookieValue {
    pub fn new(same_site: &str, secure: &str, http_only: &str) -> Self {
        Self {
            same_site: String::from(same_site),
            secure: String::from(secure),
            http_only: String::from(http_only),
        }
    }

    pub fn response(&self) -> String {
        if &self.same_site == "Lax" {
            return format!("SameSite={};", &self.same_site);
        }
        format!(
            "SameSite={}; Secure={}; HttpOnly={};",
            &self.same_site, &self.secure, &self.http_only
        )
    }
}

impl Default for CookieValue {
    fn default() -> Self {
        Self::new("", "", "")
    }
}

/// Redis connection. Used only for saving session.
#[derive(Clone)]
pub struct RedisCli {
    con: ConnectionManager,
}

impl RedisCli {
    pub async fn default() -> Self {
        let addr = env::var("REDIS").expect("redis addr not found");
        let cli = Client::open(addr).unwrap();
        let con = ConnectionManager::new(cli).await.unwrap();
        Self { con }
    }

    /// Get session if it exist.
    pub async fn get_session(&mut self, key: &str) -> Option<UserSession> {
        let s = self
            .con
            .get::<String, String>(String::from(key))
            .await
            .ok()?;
        let value = serde_json::from_str::<UserSession>(&s).ok()?;
        let value = self.set_session(key, value, false).await;
        Some(value)
    }

    /// Set new session.
    pub async fn set_session(
        &mut self,
        key: &str,
        mut value: UserSession,
        force_set: bool,
    ) -> UserSession {
        if value.is_new || force_set {
            if !force_set {
                value.not_new();
            }
            let _ = self
                .con
                .set::<String, String, String>(
                    String::from(key),
                    serde_json::to_string(&value).unwrap(),
                )
                .await;
            let _e = self
                .con
                .expire::<String, usize>(
                    String::from(key),
                    self.ttl_days(value.reg) as i64,
                )
                .await;
        }
        value
    }

    /// Create session.
    pub async fn new_session(
        &mut self,
        players: &Collection<Player>,
        cookie_value: CookieValue,
    ) -> UserSession {
        let username = create_player(players).await;
        loop {
            let s = Session::new();
            if (self.get_session(s.id()).await).is_none() {
                let value =
                    UserSession::new(&username, s.id(), false, "", cookie_value);
                return self.set_session(s.id(), value, true).await;
            }
        }
    }

    /// Returns one year ttl for registered user.
    pub fn ttl_days(&self, reg: bool) -> usize {
        let day = 60 * 60 * 24;
        if reg {
            return day * 365;
        }
        day * 2
    }
}

// #[async_trait]
impl<S> FromRequestParts<S> for UserSession
where
    AppState: FromRef<S>,
    S: Send + Sync + 'static,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppState::from_ref(state);
        let cookie: Option<TypedHeader<Cookie>> =
            parts.extract().await.unwrap_or_default();
        let Some(cookie) = cookie else {
            return Err((StatusCode::from_u16(401).unwrap(), "unauthorized"));
        };
        let prod = store.db.key.prod;
        let cookie_value = cookies(prod);
        let store = AppState::from_ref(state);
        let session_cookie = cookie.get(AXUM_SESSION_COOKIE_NAME);
        let mut redis = store.db.redis.clone();

        if let Some(session) = session_cookie {
            if let Some(session) = redis.get_session(session).await {
                return Ok(session);
            }
        }

        let session = redis
            .new_session(&store.db.mongo.players, cookie_value)
            .await;
        Ok(session)
    }
}

/// After login, this struct is returned for updating username on frontend.
#[derive(Debug, Clone, Serialize)]
pub struct VueUser {
    pub username: String,
    pub logged: bool,
}

impl From<&UserSession> for VueUser {
    fn from(user: &UserSession) -> Self {
        Self {
            username: String::from(&user.username),
            logged: user.reg,
        }
    }
}

impl From<&UserSession> for Player {
    fn from(other: &UserSession) -> Self {
        Player {
            _id: String::from(&other.username),
            reg: other.reg,
            created_at: DateTime::now(),
        }
    }
}
