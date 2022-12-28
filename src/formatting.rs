use crate::Level;
use serde::ser::{SerializeMap, Serializer};
use std::{collections::HashMap, io::Write};
use time::format_description::well_known::Rfc3339;
use tracing::{field::Visit, span::Attributes, Id};
use tracing::{Event, Subscriber};
use tracing_core::{metadata::Level as CoreLevel, Field};
use tracing_log::AsLog;
use tracing_subscriber::{
    layer::{Context, SubscriberExt},
    EnvFilter, Layer, Registry,
};

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

#[derive(Clone, Copy)]
pub struct Traceon {
    pub file: bool,
    pub module: bool,
    pub span: bool,
    pub time: bool,
    pub level: crate::Level,
}

impl Traceon {
    /// Set the writer with defaults and returns a instance of Traceon
    #[must_use]
    pub fn new() -> Traceon {
        Traceon {
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
        level: &CoreLevel,
    ) -> Result<(), std::io::Error> {
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
        std::io::stdout().write_all(&buffer)
    }
    #[must_use]
    pub fn file(&mut self, on: bool) -> &mut Self {
        self.file = on;
        self
    }
    #[must_use]
    pub fn span(&mut self, on: bool) -> &mut Self {
        self.span = on;
        self
    }
    #[must_use]
    pub fn module(&mut self, on: bool) -> &mut Self {
        self.module = on;
        self
    }
	#[must_use]
    pub fn time(&mut self, on: bool) -> &mut Self {
        self.time = on;
        self
    }
	#[must_use]
    pub fn level(&mut self, level_type: Level) -> &mut Self {
        self.level = level_type;
        self
    }

    pub fn on(self) {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(self).with(env_filter);

        // Panic if user is trying to set two global default subscribers
        tracing::subscriber::set_global_default(subscriber)
            .expect("more than one global default subscriber set");
    }

    pub fn on_thread(&self) -> tracing::subscriber::DefaultGuard {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(*self).with(env_filter);

        tracing::subscriber::set_default(subscriber)
    }

    pub fn on_with_filter(&self, filter: EnvFilter) {
        let subscriber = Registry::default().with(*self).with(filter);

        // Panic if user is trying to set two global default subscribers
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}

impl<S> Layer<S> for Traceon
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
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

            self.serialize_core_fields(&mut map_serializer, event.metadata().level())?;
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

    /// Span creation.
    /// This is the only occasion we have to store the fields attached to the span
    /// given that they might have been borrowed from the surrounding context.
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");

        // We want to inherit the fields from the parent span, if there is one.
        let mut visitor = if let Some(parent_span) = span.parent() {
            // Extensions can be used to associate arbitrary data to a span.
            // We'll use it to store our representation of its fields.
            // We create a copy of the parent visitor!
            let mut extensions = parent_span.extensions_mut();
            extensions
                .get_mut::<JsonStorage>()
                .map(|v| v.to_owned())
                .unwrap_or_default()
        } else {
            JsonStorage::default()
        };

        let mut extensions = span.extensions_mut();

        // Register all fields.
        // Fields on the new span should override fields on the parent span if there is a conflict.
        attrs.record(&mut visitor);
        // Associate the visitor with the Span for future usage via the Span's extensions
        extensions.insert(visitor);
    }

    fn on_record(&self, span: &Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(span).expect("Span not found, this is a bug");

        // Before you can associate a record to an existing Span, well, that Span has to be created!
        // We can thus rely on the invariant that we always associate a JsonVisitor with a Span
        // on creation (`new_span` method), hence it's safe to unwrap the Option.
        let mut extensions = span.extensions_mut();
        let visitor = extensions
            .get_mut::<JsonStorage>()
            .expect("Visitor not found on 'record', this is a bug");
        // Register all new fields
        values.record(visitor);
    }
}
/// `JsonStorage` will collect information about a span when it's created (`new_span` handler)
/// or when new records are attached to it (`on_record` handler) and store it in its `extensions`
/// for future retrieval from other layers interested in formatting or further enrichment.
#[derive(Clone, Debug)]
pub struct JsonStorage<'a> {
    values: HashMap<&'a str, serde_json::Value>,
}

impl<'a> JsonStorage<'a> {
    /// Get the set of stored values, as a set of keys and JSON values.
    pub fn values(&self) -> &HashMap<&'a str, serde_json::Value> {
        &self.values
    }
}

/// Get a new visitor, with an empty bag of key-value pairs.
impl Default for JsonStorage<'_> {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

/// Taken verbatim from tracing-subscriber
impl Visit for JsonStorage<'_> {
    /// Visit a signed 64-bit integer value.
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }

    /// Visit an unsigned 64-bit integer value.
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }

    /// Visit a 64-bit floating point value.
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }

    /// Visit a boolean value.
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }

    /// Visit a string value.
    fn record_str(&mut self, field: &Field, value: &str) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            // Skip fields that are actually log metadata that have already been handled
            name if name.starts_with("log.") => (),
            name if name.starts_with("r#") => {
                self.values
                    .insert(&name[2..], serde_json::Value::from(format!("{:?}", value)));
            }
            name => {
                self.values
                    .insert(name, serde_json::Value::from(format!("{:?}", value)));
            }
        };
    }
}
