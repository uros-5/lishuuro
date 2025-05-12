use std::sync::Arc;

use tokio::{
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    time,
};

use super::game::GameMessage;

pub async fn clock_task(game: Sender<GameMessage>) -> mpsc::Sender<ClockMessage> {
    let (sender, mut recv) = mpsc::channel::<ClockMessage>(20);

    let interval =
        Arc::new(Mutex::new(time::interval(time::Duration::from_secs(5))));
    let interval_loop = interval.clone();
    tokio::spawn(async move {
        while let Some(msg) = recv.recv().await {
            match msg {
                ClockMessage::IncreaseInterval(duration) => {
                    let duration = time::Duration::from_millis(duration);
                    *interval.lock().await = time::interval(duration);
                }
                ClockMessage::StopClock => {
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        loop {
            interval_loop.lock().await.tick().await;
            if let Err(_) = game.send(GameMessage::CheckClock).await {
                break;
            }
        }
    });

    sender
}

pub enum ClockMessage {
    IncreaseInterval(u64),
    StopClock,
}
