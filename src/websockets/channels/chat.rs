use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub user: String,
    #[serde(serialize_with = "to_rfc3339")]
    pub time: DateTime<Local>,
    pub message: String,
}

impl ChatMessage {
    pub fn update(&mut self, user: &String) {
        self.user = user.to_string();

        chrono::offset::Local::now();
        self.time = chrono::offset::Local::now();
    }
}

fn to_rfc3339<S>(time: &DateTime<Local>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let time = time.to_rfc3339();
    s.serialize_str(&time)
}
