use chrono::{DateTime, Utc};
use log::error;
use rsntp::AsyncSntpClient;
use std::time::{Duration, Instant};
use tokio::time::interval;

use crate::Time;

/// Produces current timestamps with high precision
pub struct SyncedTime {
    datetime: DateTime<Utc>,
    instant: Instant,
}

impl Default for SyncedTime {
    fn default() -> Self {
        Self::new(Utc::now())
    }
}

impl SyncedTime {
    fn new(datetime: DateTime<Utc>) -> Self {
        Self {
            datetime,
            instant: Instant::now(),
        }
    }

    /// Returns the current timestamp in microseconds
    pub fn current_timestamp(&self) -> u128 {
        self.datetime.timestamp_micros() as u128 + self.instant.elapsed().as_micros()
    }
}

/// A background thread which produces synchronized datetime objects
pub async fn synchronize(time: Time) {
    let client = AsyncSntpClient::new();

    // time is re-synced every 5 minutes
    let mut interval = interval(Duration::from_secs(60 * 5));

    loop {
        interval.tick().await;

        match client.synchronize("pool.ntp.org").await {
            Ok(result) => {
                match result.datetime().into_chrono_datetime() {
                    Ok(new_time) => {
                        // store the new time
                        *time.lock().await = SyncedTime::new(new_time);
                        continue;
                    }
                    Err(error) => error!("Failed to convert datetime: {}", error),
                }
            }
            Err(error) => error!("Failed to synchronize time: {}", error),
        }

        // retry after 20 seconds
        interval.reset_after(Duration::from_secs(20));
    }
}
