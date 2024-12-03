use crate::logger::{Level, Logger};

pub(crate) struct LoggerBuilder {
    pub(crate) level: Level,
    pub(crate) local_timestamp: bool,
    constant_fields: indexmap::IndexMap<String, String>,
}

impl LoggerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            level: Level::Info,
            local_timestamp: true,
            constant_fields: indexmap::IndexMap::new(),
        }
    }

    pub(crate) fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub(crate) fn with_field(mut self, name: &str, value: &str) -> Self {
        self.constant_fields.insert(name.to_string(), value.to_string());
        self
    }

    pub(crate) fn constant_fields(&self) -> indexmap::IndexMap<String, serde_json::Value> {
        self.constant_fields
            .iter()
            .map(|(k,v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect()
    }

    pub(crate) fn with_local_timestamp(mut self, use_local_timestamp: bool) -> Self {
        self.local_timestamp = use_local_timestamp;
        self
    }

    pub(crate) fn build(&self) -> Logger {
        Logger::new(self)
    }
}
