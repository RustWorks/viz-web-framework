use std::{env, fmt};

use serde::{Deserialize, Serialize};

/// Current Env
#[derive(Deserialize, Serialize)]
pub enum Env {
    /// Development
    Dev,
    /// Production
    Prod,
    /// Test
    Test,
}

impl Default for Env {
    fn default() -> Env {
        Env::Dev
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Dev => Self::DEV,
            Self::Prod => Self::PROD,
            Self::Test => Self::TEST,
        })
    }
}

impl Env {
    /// Viz environment name
    pub const NAME: &'static str = "VIZ_ENV";
    /// Development mode
    pub const DEV: &'static str = "development";
    /// Production mode
    pub const PROD: &'static str = "production";
    /// Test mode
    pub const TEST: &'static str = "test";

    /// Gets current mode
    pub fn get() -> Env {
        env::var(Self::NAME).map(From::from).unwrap_or_default()
    }
}

impl From<String> for Env {
    fn from(mut s: String) -> Env {
        s = s.trim().to_lowercase();

        if Self::DEV.starts_with(&s) {
            return Env::Dev;
        }

        if Self::PROD.starts_with(&s) {
            return Env::Prod;
        }

        if Self::TEST == s {
            return Env::Test;
        }

        return Env::Dev;
    }
}

impl ToString for Env {
    fn to_string(&self) -> String {
        match *self {
            Self::Dev => Self::DEV.split_at(3),
            Self::Prod => Self::PROD.split_at(4),
            Self::Test => Self::TEST.split_at(4),
        }
        .0
        .to_string()
    }
}
