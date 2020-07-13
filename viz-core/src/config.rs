use std::{env, fmt, fs, io, path::Path, sync::Arc};

use blocking::{block_on, unblock};

use serde::{Deserialize, Serialize};

use toml::{
    self,
    value::{Map, Value},
};

use viz_utils::{futures::future::BoxFuture, log, serde::json};

use crate::{Context, Error, Extract, Result, PAYLOAD_LIMIT};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub limits: Limits,

    #[serde(skip_deserializing)]
    pub env: Env,

    #[serde(default)]
    pub extras: Map<String, Value>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            limits: Limits::default(),
            env: Env::default(),
            extras: Map::new(),
        }
    }

    pub async fn load() -> Result<Config> {
        let path = env::current_dir()?;

        let e = Env::get();

        let config_path = path.join("config").join(e.to_string() + ".toml");

        let mut config = if config_path.exists() {
            block_on(async { Ok::<_, Error>(toml::from_str(&fs::read_to_string(config_path)?)?) })
                .unwrap_or_default()
        } else {
            Config::new()
        };

        config.env = e;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            env: Env::default(),
            extras: Map::default(),
        }
    }
}

/// Body payload data limits
#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    #[serde(default = "form_limit")]
    pub form: usize,
    #[serde(default = "json_limit")]
    pub json: usize,
    #[serde(default = "multipart_limit")]
    pub multipart: usize,
}

fn form_limit() -> usize {
    PAYLOAD_LIMIT
}

fn json_limit() -> usize {
    PAYLOAD_LIMIT * 8
}

fn multipart_limit() -> usize {
    PAYLOAD_LIMIT * 16
}

impl Default for Limits {
    fn default() -> Limits {
        Self {
            form: form_limit(),
            json: json_limit(),
            multipart: multipart_limit(),
        }
    }
}

impl Limits {
    fn new() -> Self {
        Self::default()
    }
}

impl Extract for Arc<Config> {
    type Error = Error;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { Ok(cx.config()) })
    }
}

pub trait ContextExt {
    fn config(&self) -> Arc<Config>;
}

impl ContextExt for Context {
    fn config(&self) -> Arc<Config> {
        self.extensions()
            .get::<Arc<Config>>()
            .cloned()
            .unwrap_or_default()
    }
}

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
    pub const NAME: &'static str = "VIZ_ENV";
    pub const DEV: &'static str = "development";
    pub const PROD: &'static str = "production";
    pub const TEST: &'static str = "test";

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
