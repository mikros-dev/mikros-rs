pub mod builder;
mod middleware;

use std::str::FromStr;

use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    layer::{Layered, SubscriberExt},
    reload::Handle,
    EnvFilter, Registry,
};

use crate::logger::builder::LoggerBuilder;
use crate::logger::middleware::{Layer, LayerBuilder};

pub struct Logger {
    reload: Handle<EnvFilter, Layered<Layer, Registry>>,
}

#[derive(Clone)]
pub enum Level {
    Debug,
    Info,
    Warning,
    Error,
}

impl From<Level> for tracing::Level {
    fn from(level: Level) -> Self {
        match level {
            Level::Debug => tracing::Level::DEBUG,
            Level::Info => tracing::Level::INFO,
            Level::Warning => tracing::Level::WARN,
            Level::Error => tracing::Level::ERROR,
        }
    }
}

impl FromStr for Level {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(Level::Debug),
            "info" => Ok(Level::Info),
            "warning" => Ok(Level::Warning),
            "error" => Ok(Level::Error),
            _ => Err(format!("unknown log level {}", s)),
        }
    }
}

impl Logger {
    pub(crate) fn new(builder: &LoggerBuilder) -> Self {
        let level: tracing::Level = builder.level.clone().into();
        let env_filter = EnvFilter::new(level.to_string());
        let (filter_layer, reload) = tracing_subscriber::reload::Layer::new(env_filter);

        // Do not initialize the global subscriber if we're running tests.
        if !cfg!(test) {
            tracing_subscriber::registry()
                .with(
                    LayerBuilder::new()
                        .with_local_timestamp(builder.local_timestamp)
                        .with_constant_fields(builder.constant_fields())
                        .build(),
                )
                .with(filter_layer)
                .init();
        }

        Self { reload }
    }

    pub fn debug(&self, message: &str) {
        self.logf(Level::Debug, message, None)
    }

    pub fn info(&self, message: &str) {
        self.logf(Level::Info, message, None)
    }

    pub fn warning(&self, message: &str) {
        self.logf(Level::Warning, message, None)
    }

    pub fn error(&self, message: &str) {
        self.logf(Level::Error, message, None)
    }

    pub fn debugf(&self, message: &str, fields: serde_json::Value) {
        self.logf(Level::Debug, message, Some(fields))
    }

    pub fn infof(&self, message: &str, fields: serde_json::Value) {
        self.logf(Level::Info, message, Some(fields))
    }

    pub fn warningf(&self, message: &str, fields: serde_json::Value) {
        self.logf(Level::Warning, message, Some(fields))
    }

    pub fn errorf(&self, message: &str, fields: serde_json::Value) {
        self.logf(Level::Error, message, Some(fields))
    }

    fn logf(&self, level: Level, message: &str, data: Option<serde_json::Value>) {
        let mut fields = indexmap::IndexMap::new();
        if let Some(serde_json::Value::Object(data_map)) = data {
            fields.extend(data_map);
        }

        let call_fields = serde_json::to_string(&fields).unwrap();
        match level {
            Level::Debug => tracing::debug!(%call_fields, message = %message),
            Level::Info => tracing::info!(%call_fields, message = %message),
            Level::Warning => tracing::warn!(%call_fields, message = %message),
            Level::Error => tracing::error!(%call_fields, message = %message),
        }
    }

    pub fn change_level(&self, level: Level) {
        let level: tracing::Level = level.into();
        let _ = self
            .reload
            .modify(|f| *f = EnvFilter::new(level.to_string()));
    }
}
