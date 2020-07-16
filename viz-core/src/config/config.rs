use std::{env, fs, sync::Arc};

use blocking::block_on;
use serde::{Deserialize, Serialize};
use toml::{
    self,
    value::{Map, Value},
};

use viz_utils::futures::future::BoxFuture;

use crate::{Context, Error, Extract, Result};

use super::{Cookies, Env, Limits};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub limits: Limits,

    #[serde(default)]
    pub cookies: Cookies,

    #[serde(skip_deserializing)]
    pub env: Env,

    #[serde(default)]
    pub extras: Map<String, Value>,
}

impl Config {
    pub async fn load() -> Result<Config> {
        let path = env::current_dir()?;

        let e = Env::get();

        let config_path = path.join("config").join(e.to_string() + ".toml");

        let mut config = if config_path.exists() {
            block_on(async { Ok::<_, Error>(toml::from_str(&fs::read_to_string(config_path)?)?) })
                .unwrap_or_default()
        } else {
            Config::default()
        };

        config.env = e;

        dbg!(&config);
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            cookies: Cookies::default(),
            env: Env::default(),
            extras: Map::default(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
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
