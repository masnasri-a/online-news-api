use std::sync::Arc;
use chrono::Utc;
use dashmap::DashMap;

use crate::config::Config;
use crate::domain::tier::SubscriptionTier;
use crate::errors::AppError;

/// Tracks per-user, per-hour request counts.
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u64,
    hour: u32,  // hour of day (0–23) for hourly reset
    day: u32,   // day of year for cross-day detection
}

/// In-memory rate limiter with hourly windows per user+tier.
#[derive(Clone)]
pub struct RateLimiter {
    entries: Arc<DashMap<String, RateLimitEntry>>,
    config: Config,
}

impl RateLimiter {
    pub fn new(config: Config) -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Check whether the request is allowed. Returns `(limit, remaining)`
    /// on success, or an `AppError::RateLimitExceeded` on failure.
    pub fn check(&self, user: &str, tier: &SubscriptionTier) -> Result<(u64, u64), AppError> {
        let now = Utc::now();
        let current_hour = now.format("%H").to_string().parse::<u32>().unwrap_or(0);
        let current_day = now.format("%j").to_string().parse::<u32>().unwrap_or(0);
        let limit = tier.hourly_limit(&self.config);

        let key = format!("{}:{}", user, tier.name());
        let mut entry = self.entries.entry(key).or_insert(RateLimitEntry {
            count: 0,
            hour: current_hour,
            day: current_day,
        });

        // Reset on new hour or new day
        if entry.hour != current_hour || entry.day != current_day {
            entry.count = 0;
            entry.hour = current_hour;
            entry.day = current_day;
        }

        if entry.count >= limit {
            let reset_at = (now + chrono::Duration::hours(1))
                .format("%Y-%m-%dT%H:00:00Z")
                .to_string();
            return Err(AppError::RateLimitExceeded {
                tier: tier.name().to_string(),
                limit,
                reset_at,
            });
        }

        entry.count += 1;
        let remaining = limit - entry.count;
        Ok((limit, remaining))
    }

    /// Get the hourly reset timestamp for headers.
    pub fn reset_time() -> String {
        let now = Utc::now();
        (now + chrono::Duration::hours(1))
            .format("%Y-%m-%dT%H:00:00Z")
            .to_string()
    }
}
