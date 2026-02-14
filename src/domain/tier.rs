use crate::config::Config;

/// Subscription tiers matching RapidAPI plan names.
#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionTier {
    Basic,    // Free — 5 req/hour
    Pro,      // $49/mo — 100 req/hour
    Ultra,    // $99/mo — 1,000 req/hour
    Mega,     // $199/mo — 10,000 req/hour
}

impl SubscriptionTier {
    /// Parse from the `X-RapidAPI-Subscription` header value.
    pub fn from_header(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "PRO" => Self::Pro,
            "ULTRA" => Self::Ultra,
            "MEGA" | "CUSTOM" => Self::Mega,
            _ => Self::Basic,
        }
    }

    /// Display name for external communication.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Pro => "pro",
            Self::Ultra => "ultra",
            Self::Mega => "mega",
        }
    }

    /// Hourly request limit for this tier.
    pub fn hourly_limit(&self, config: &Config) -> u64 {
        match self {
            Self::Basic => config.rate_limit_basic,
            Self::Pro => config.rate_limit_pro,
            Self::Ultra => config.rate_limit_ultra,
            Self::Mega => config.rate_limit_mega,
        }
    }

    /// Maximum page size allowed for this tier.
    pub fn max_page_size(&self) -> u64 {
        match self {
            Self::Basic => 10,
            Self::Pro => 25,
            Self::Ultra => 50,
            Self::Mega => 100,
        }
    }

    /// Whether this tier receives full article content.
    pub fn has_full_content(&self) -> bool {
        !matches!(self, Self::Basic)
    }

    /// Whether this tier receives NLP entity data.
    pub fn has_entities(&self) -> bool {
        matches!(self, Self::Ultra | Self::Mega)
    }

    /// Price label for error messages.
    pub fn price_label(&self) -> &'static str {
        match self {
            Self::Basic => "Free",
            Self::Pro => "$49/mo",
            Self::Ultra => "$99/mo",
            Self::Mega => "$199/mo",
        }
    }
}
