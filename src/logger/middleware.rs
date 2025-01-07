use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

use tracing::field::Field;
use tracing::Subscriber;

pub(crate) struct LayerBuilder {
    local_timestamp: bool,
    constant_fields: indexmap::IndexMap<String, serde_json::Value>,
}

impl LayerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            local_timestamp: true,
            constant_fields: indexmap::IndexMap::new(),
        }
    }

    pub(crate) fn with_local_timestamp(mut self, use_local_timestamp: bool) -> Self {
        self.local_timestamp = use_local_timestamp;
        self
    }

    pub(crate) fn with_constant_fields(
        mut self,
        fields: indexmap::IndexMap<String, serde_json::Value>,
    ) -> Self {
        self.constant_fields = fields;
        self
    }

    pub(crate) fn build(&self) -> Layer {
        Layer {
            local_timestamp: self.local_timestamp,
            constant_fields: self.constant_fields.clone(),
        }
    }
}

pub(crate) struct Layer {
    local_timestamp: bool,
    constant_fields: indexmap::IndexMap<String, serde_json::Value>,
}

impl<S> tracing_subscriber::Layer<S> for Layer
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut now = chrono::Local::now().to_rfc3339();

        if !self.local_timestamp {
            now = chrono::Utc::now().to_rfc3339()
        }

        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        let mut output = indexmap::IndexMap::new();
        output.insert("timestamp".to_string(), serde_json::Value::String(now));
        output.insert(
            "level".to_string(),
            serde_json::Value::String(event.metadata().level().to_string()),
        );

        let message = visitor.0.remove("message").unwrap();
        output.insert("message".to_string(), message);

        // user constant fields
        for (k, v) in &self.constant_fields {
            output.insert(k.to_string(), v.clone());
        }

        // call fields
        if let Some(call_fields) = visitor.0.remove("call_fields") {
            if let Some(fields) = call_fields.as_object() {
                for (k, v) in fields {
                    output.insert(k.to_string(), v.clone());
                }
            }
        }

        println!("{}", serde_json::to_string(&output).unwrap());
    }
}

#[derive(Default)]
pub(crate) struct FieldVisitor(HashMap<String, serde_json::Value>);

impl tracing::field::Visit for FieldVisitor {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().to_string(), value.into());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_string(), value.into());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_string(), value.into());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_string(), value.into());
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        match serde_json::from_str::<serde_json::Value>(value) {
            Ok(v) => self.0.insert(field.name().to_string(), v),
            Err(_) => self
                .0
                .insert(field.name().to_string(), value.to_string().into()),
        };
    }

    fn record_error(&mut self, field: &Field, value: &(dyn Error + 'static)) {
        self.0
            .insert(field.name().to_string(), value.to_string().into());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        let v = format!("{:?}", value);
        match serde_json::from_str::<serde_json::Value>(&v) {
            Ok(value) => self.0.insert(field.name().to_string(), value),
            Err(_) => self.0.insert(field.name().to_string(), v.into()),
        };
    }
}
