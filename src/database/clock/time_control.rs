use chrono::{DateTime, Duration, FixedOffset, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use shuuro::Color;
use typeshare::typeshare;

use crate::database::serde_helpers::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[typeshare]
pub struct TimeControl {
    #[typeshare(serialized_as = "String")]
    pub last_click: DateTime<FixedOffset>,
    #[serde(serialize_with = "duration_to_array")]
    #[serde(deserialize_with = "array_to_duration")]
    #[typeshare(serialized_as = "[u8; 2]")]
    pub clocks: [Duration; 2],
    #[serde(skip)]
    pub stage: u8,
    #[serde(skip)]
    pub incr: i64,
}

impl TimeControl {
    /// Create new time control.
    pub fn new(time: i64, incr: i64) -> Self {
        let duration = Duration::seconds(time * 60 + incr);
        let last_click = Utc::now().into();

        Self {
            clocks: [duration, duration],
            stage: 0,
            incr,
            last_click,
        }
    }

    pub fn update_stage(&mut self, stage: u8) {
        self.stage = stage;
        self.last_click = Utc::now().into();
    }

    pub fn play(&mut self, color: usize) -> Option<[u64; 2]> {
        let duration = self.current_duration(color)?;
        self.update_last_click(color, duration);
        let ms = [
            self.clocks[0].num_milliseconds() as u64,
            self.clocks[1].num_milliseconds() as u64,
        ];
        Some(ms)
    }

    pub fn current_duration(&self, color: usize) -> Option<Duration> {
        let elapsed = self.elapsed();
        let duration = self.clocks[color].checked_sub(&elapsed)?;
        if duration.num_seconds() < 0 {
            return None;
        }
        Some(duration)
    }

    fn update_last_click(&mut self, color: usize, current: Duration) {
        if self.stage == 0 {
            return;
        }
        let duration = current.checked_add(&Duration::seconds(self.incr));
        match duration {
            Some(duration) => {
                self.clocks[color] = duration;

                if self.stage < 3 {
                    self.last_click = Utc::now().into();
                }
            }
            None => self.clocks[color] = Duration::seconds(0),
        }
    }

    pub fn select(&mut self, me: Color) -> [TimeDelta; 2] {
        self.stage = 3;
        self.play(me.index());
        self.stage = 0;
        self.clocks.clone()
    }

    pub fn set_to_zero(&mut self, player: Color) {
        self.clocks[player.index()] = Duration::seconds(0);
    }

    fn elapsed(&self) -> Duration {
        let now: DateTime<FixedOffset> = Utc::now().into();
        now - self.last_click
    }
}
