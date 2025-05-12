use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use reqwest::{Client, Url};

use crate::lichess::login_helpers::base64_encode;

use super::{
    curr_url,
    login_helpers::{create_challenge, create_verifier},
    LoginData, PostLoginToken, Token,
};
use rand::Rng;

/// Start of login process.
pub fn login_url(login_state: &str, prod: bool) -> (Url, String) {
    let url = "https://lichess.org/oauth?";
    let verifier: String = create_verifier();
    let challenge: String = create_challenge(&verifier);
    let mut final_url = Url::parse(url).unwrap();
    let r = format!("{}/callback", curr_url(prod).0);

    let queries = [
        ("state", login_state),
        ("response_type", "code"),
        ("client_id", "lishuuro"),
        ("redirect_uri", &r),
        ("code_challenge", &challenge[..]),
        ("code_challenge_method", "S256"),
    ];

    for i in queries {
        final_url.query_pairs_mut().append_pair(i.0, i.1);
    }

    (final_url, verifier)
}

/// If anon then generate random username.
pub fn random_username() -> String {
    let random_bytes = rand::rng().random::<[u8; 6]>();

    format!(
        "Anon-{}",
        base64_encode(random_bytes).replace(['+', '/', '='], "")
    )
}

/// Getting lichess token.
pub async fn get_lichess_token(
    code: &String,
    code_verifier: &String,
    prod: bool,
) -> Result<Token, LichessError> {
    let url = "https://lichess.org/api/token";
    let body = PostLoginToken::new(code_verifier, code);
    let body = body.to_json(prod);
    let client = Client::default();
    let req = client
        .post(url)
        .json(&body)
        .send()
        .await
        .or(Err(LichessError::NoToken))?;

    req.json::<Token>().await.or(Err(LichessError::NoToken))
}

/// If user exist then we have login data.
pub async fn get_lichess_user(token: String) -> Result<String, LichessError> {
    let url = "https://lichess.org/api/account";
    let client = Client::default();
    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .or(Err(LichessError::NoUsername))?;
    let json = res
        .json::<LoginData>()
        .await
        .or(Err(LichessError::NoUsername))?;
    Ok(json.username)
}

pub enum LichessError {
    NoToken,
    NoUsername,
}

impl IntoResponse for LichessError {
    fn into_response(self) -> Response {
        let body = match self {
            LichessError::NoToken => "no lichess token",
            LichessError::NoUsername => "account rejected",
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
