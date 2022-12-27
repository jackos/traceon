use crate::JsonStorage;
use crate::Level;
use crate::StorageLayer;

use serde::ser::{SerializeMap, Serializer};
use serde_json::Value;
use std::io::Write;
use time::format_description::well_known::Rfc3339;
use tracing::{Event, Subscriber};
use tracing_core::metadata::Level as CoreLevel;
use tracing_log::AsLog;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;
use tracing_subscriber::{EnvFilter, Registry};

/// Convert from log levels to an u16 for easy filtering
fn level_to_u16(level: &CoreLevel) -> u16 {
    match level.as_log() {
        log::Level::Error => 50,
        log::Level::Warn => 40,
        log::Level::Info => 30,
        log::Level::Debug => 20,
        log::Level::Trace => 10,
    }
}

#[derive(Copy, Clone)]
pub struct Traceon<
    W: for<'a> MakeWriter<'a> + 'static + std::marker::Sync + std::marker::Send + Copy + Clone,
> {
    pub writer: W,
    pub file: bool,
    pub module: bool,
    pub span: bool,
    pub time: bool,
    pub level: crate::Level,
}

impl<
        W: for<'a> MakeWriter<'a> + 'static + std::marker::Sync + std::marker::Send + Copy + Clone,
    > Traceon<W>
{
    /// Set the writer with defaults and returns a instance of Traceon
    pub fn new(writer: W) -> Traceon<W> {
        Traceon {
            writer,
            file: true,
            span: true,
            time: true,
            module: false,
            level: crate::Level::Number,
        }
    }

    /// Create a new `FormattingLayer`.
    fn serialize_core_fields(
        &self,
        map_serializer: &mut impl SerializeMap<Error = serde_json::Error>,
        message: &str,
        level: &CoreLevel,
    ) -> Result<(), std::io::Error> {
        map_serializer.serialize_entry("message", &message)?;
        match self.level {
            Level::Text => {
                map_serializer.serialize_entry("level", &level.to_string())?;
            }
            Level::Number => {
                map_serializer.serialize_entry("level", &level_to_u16(level))?;
            }
            Level::Off => (),
        }
        if self.time {
            if let Ok(time) = &time::OffsetDateTime::now_utc().format(&Rfc3339) {
                map_serializer.serialize_entry("time", time)?;
            }
        }
        Ok(())
    }

    fn emit(&self, mut buffer: Vec<u8>) -> Result<(), std::io::Error> {
        buffer.write_all(b"\n")?;
        self.writer.make_writer().write_all(&buffer)
    }
    pub fn file(&mut self, on: bool) -> &mut Self {
        self.file = on;
        self
    }
    pub fn span(&mut self, on: bool) -> &mut Self {
        self.span = on;
        self
    }
    pub fn module(&mut self, on: bool) -> &mut Self {
        self.module = on;
        self
    }
    pub fn time(&mut self, on: bool) -> &mut Self {
        self.time = on;
        self
    }
    pub fn level(&mut self, level_type: Level) -> &mut Self {
        self.level = level_type;
        self
    }

    pub fn on(&self) {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default()
            .with(StorageLayer)
            .with(*self)
            .with(env_filter);

        // Panic if user is trying to set two global default subscribers
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    pub fn on_with_filter(&self, filter: EnvFilter) {
        let subscriber = Registry::default()
            .with(StorageLayer)
            .with(*self)
            .with(filter);

        // Panic if user is trying to set two global default subscribers
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}

/// flatten the message, use target if no message exists
fn format_event_message(event: &Event, event_visitor: &JsonStorage<'_>) -> String {
    // Extract the "message" field, if provided. Fallback to the target, if missing.
    event_visitor
        .values()
        .get("message")
        .map(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
        .flatten()
        .unwrap_or_else(|| event.metadata().target())
        .to_owned()
}

impl<S, W> Layer<S> for Traceon<W>
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    W: for<'a> MakeWriter<'a> + 'static + std::marker::Sync + std::marker::Send + Clone + Copy,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Events do not necessarily happen in the context of a span, hence lookup_current
        // returns an `Option<SpanRef<_>>` instead of a `SpanRef<_>`.
        let current_span = ctx.lookup_current();

        let mut event_visitor = JsonStorage::default();
        event.record(&mut event_visitor);

        // Opting for a closure to use the ? operator and get more linear code.
        let format = || {
            let mut buffer = Vec::new();

            let mut serializer = serde_json::Serializer::new(&mut buffer);
            let mut map_serializer = serializer.serialize_map(None)?;

            let message = format_event_message(event, &event_visitor);
            self.serialize_core_fields(&mut map_serializer, &message, event.metadata().level())?;
            // Add file and line number to the json
            let metadata = event.metadata();
            if self.span {
                if let Some(span) = &current_span {
                    map_serializer.serialize_entry("span", span.metadata().name())?;
                }
            }

            if self.module {
                map_serializer
                    .serialize_entry("module", metadata.module_path().unwrap_or_default())?;
            }

            if self.file {
                map_serializer.serialize_entry(
                    "file",
                    &format!(
                        "{}:{}",
                        metadata.file().unwrap_or_default(),
                        metadata.line().unwrap_or_default()
                    ),
                )?;
            }

            // Add fields associated with the event, expect the message we already used.
            for (key, value) in event_visitor.values().iter()
            // .filter(|(&key, _)| key != "message" && !TRACEON_RESERVED.contains(&key))
            {
                map_serializer.serialize_entry(key, value)?;
            }

            // Add all the fields from the current span, if we have one.
            if let Some(span) = &current_span {
                let extensions = span.extensions();
                if let Some(visitor) = extensions.get::<JsonStorage>() {
                    for (key, value) in visitor.values() {
                        map_serializer.serialize_entry(key, value)?;
                    }
                }
            }
            map_serializer.end()?;
            Ok(buffer)
        };

        let result: std::io::Result<Vec<u8>> = format();
        if let Ok(formatted) = result {
            let _ = self.emit(formatted);
        }
    }
}
