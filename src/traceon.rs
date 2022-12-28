use serde::ser::{SerializeMap, Serializer};
use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, Mutex},
};
use time::format_description::well_known::Rfc3339;
use tracing::{field::Visit, span::Attributes, Event, Id, Subscriber};
use tracing_core::Field;
use tracing_log::AsLog;
use tracing_subscriber::{
    layer::{Context, SubscriberExt},
    EnvFilter, Layer, Registry,
};

#[derive(Clone)]
pub struct Traceon {
    pub writer: Arc<Mutex<dyn Write + Sync + Send>>,
    pub filter: Arc<EnvFilter>,
    pub file: bool,
    pub module: bool,
    pub span: bool,
    pub time: bool,
    pub concat: String,
    pub level: LevelFormat,
    pub key_case: KeyCase,
}

#[derive(Clone)]
pub enum KeyCase {
    Camel,
    Pascal,
    Snake,
    None,
}

#[derive(Copy, Clone)]
pub enum LevelFormat {
    Off,
    Text,
    Number,
}

impl Default for Traceon {
    #[must_use]
    fn default() -> Traceon {
        let filter =
            Arc::new(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));
        Traceon {
            writer: Arc::new(Mutex::new(std::io::stdout())),
            filter,
            concat: "".into(),
            file: true,
            span: true,
            time: true,
            module: true,
            key_case: KeyCase::None,
            level: crate::LevelFormat::Number,
        }
    }
}

impl Traceon {
    /// Set the writer with defaults and returns a instance of Traceon
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
    pub fn concat(&mut self, concat: &str) -> &mut Self {
        self.concat = concat.to_string();
        self
    }
    #[must_use]
    pub fn time(&mut self, on: bool) -> &mut Self {
        self.time = on;
        self
    }
    #[must_use]
    pub fn level(&mut self, level_type: LevelFormat) -> &mut Self {
        self.level = level_type;
        self
    }
    #[must_use]
    pub fn writer(&mut self, writer: impl Write + Send + Sync + 'static) -> &mut Self {
        self.writer = Arc::new(Mutex::new(writer));
        self
    }
    #[must_use]
    pub fn key_case(&mut self, key_case: KeyCase) -> &mut Self {
        self.key_case = key_case;
        self
    }

    pub fn on(&self) {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(self.clone()).with(env_filter);

        // Panic if user is trying to set two global default subscribers
        tracing::subscriber::set_global_default(subscriber)
            .expect("more than one global default subscriber set");
    }

    /// Use the defaults and set the global default subscriber
    pub fn try_on(&self) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(self.clone()).with(env_filter);

        tracing::subscriber::set_global_default(subscriber)
    }

    pub fn on_thread(&self) -> tracing::subscriber::DefaultGuard {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(self.clone()).with(env_filter);

        tracing::subscriber::set_default(subscriber)
    }
}

impl<S> Layer<S> for Traceon
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let current_span = ctx.lookup_current();

        let mut event_visitor = JsonStorage::new(self.concat.clone());
        event.record(&mut event_visitor);

        // Closure allows use of the ? syntax
        let format = || {
            let (level_key, file_key, module_key, timestamp_key) = match self.key_case {
                KeyCase::Pascal => ("Level", "File", "Module", "Timestamp"),
                _ => ("level", "file", "module", "timestamp"),
            };

            let mut buffer = Vec::new();

            let mut serializer = serde_json::Serializer::new(&mut buffer);
            let mut map_serializer = serializer.serialize_map(None)?;

            let metadata = event.metadata();
            match self.level {
                LevelFormat::Text => {
                    map_serializer.serialize_entry(level_key, &metadata.level().to_string())?;
                }
                LevelFormat::Number => {
                    let number = match metadata.level().as_log() {
                        log::Level::Error => 50u16,
                        log::Level::Warn => 40,
                        log::Level::Info => 30,
                        log::Level::Debug => 20,
                        log::Level::Trace => 10,
                    };

                    map_serializer.serialize_entry(level_key, &number)?;
                }
                LevelFormat::Off => (),
            }
            if self.time {
                if let Ok(time) = &time::OffsetDateTime::now_utc().format(&Rfc3339) {
                    map_serializer.serialize_entry(timestamp_key, time)?;
                }
            }

            if self.module {
                map_serializer
                    .serialize_entry(module_key, metadata.module_path().unwrap_or_default())?;
            }

            if self.file {
                map_serializer.serialize_entry(
                    file_key,
                    &format!(
                        "{}:{}",
                        metadata.file().unwrap_or_default(),
                        metadata.line().unwrap_or_default()
                    ),
                )?;
            }

            // Add all the fields from the current event.
            for (key, value) in event_visitor.values.iter() {
                let key = match self.key_case {
                    KeyCase::Snake => snake(key),
                    KeyCase::Pascal => pascal(key),
                    KeyCase::Camel => camel(key),
                    KeyCase::None => key.to_string(),
                };

                map_serializer.serialize_entry(&key, value)?;
            }

            // Add all the fields from the current span, if we have one.
            if let Some(span) = &current_span {
                let extensions = span.extensions();
                if let Some(visitor) = extensions.get::<JsonStorage>() {
                    for (key, value) in &visitor.values {
                        let key = match self.key_case {
                            KeyCase::Snake => snake(key),
                            KeyCase::Pascal => pascal(key),
                            KeyCase::Camel => camel(key),
                            KeyCase::None => key.to_string(),
                        };

                        map_serializer.serialize_entry(&key, value)?;
                    }
                }
            }
            map_serializer.end()?;
            Ok(buffer)
        };

        let result: std::io::Result<Vec<u8>> = format();
        if let Err(e) = &result {
            dbg!(e);
        }
        if let Ok(mut formatted) = result {
            formatted.write_all(b"\n").unwrap();
            self.writer.lock().unwrap().write_all(&formatted).unwrap();
        }
    }

    /// This is the only occasion we have to store the fields attached to the span
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        // We want to inherit the fields from the parent span, if there is one.
        let mut visitor = if let Some(parent_span) = span.parent() {
            let mut extensions = parent_span.extensions_mut();
            let mut storage = extensions
                .get_mut::<JsonStorage>()
                .map(|v| v.to_owned())
                .unwrap_or_default();
            if self.span {
                if let Some(orig) = storage.values.insert(
                    "span",
                    serde_json::Value::from(format!(
                        "{}::{}",
                        parent_span.metadata().name(),
                        span.metadata().name()
                    )),
                ) {
                    if self.concat != "" {
                        storage.values.insert(
                            "span",
                            serde_json::Value::from(format!(
                                "{}{}{}",
                                orig.as_str().unwrap_or(""),
                                self.concat,
                                span.metadata().name()
                            )),
                        );
                    }
                };
            }
            storage
        } else {
            JsonStorage::new(self.concat.clone())
        };

        let mut extensions = span.extensions_mut();
        // Fields on the new span should override fields on the parent span if there is a conflict.
        attrs.record(&mut visitor);
        // Associate the visitor with the Span for future usage via the Span's extensions
        extensions.insert(visitor);
    }

    fn on_record(&self, span: &Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(span).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        let visitor = extensions
            .get_mut::<JsonStorage>()
            .expect("Visitor not found on 'record', this is a bug");
        values.record(visitor);
    }
}

/// Responsible for storing fields as a set of keys and JSON values when visiting a span
#[derive(Clone, Debug, Default)]
pub struct JsonStorage<'a> {
    pub values: HashMap<&'a str, serde_json::Value>,
    pub concat: String,
}

impl<'a> JsonStorage<'a> {
    pub fn new(concat: String) -> Self {
        JsonStorage {
            values: HashMap::new(),
            concat,
        }
    }
}

fn snake(key: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in key.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

fn pascal(key: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    for ch in key.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(ch);
        }
    }
    pascal
}

fn camel(pascal: &str) -> String {
    pascal[..1].to_ascii_lowercase() + &pascal[1..]
}

impl Visit for JsonStorage<'_> {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.values
            .insert(field.name(), serde_json::Value::from(value));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if let Some(orig) = self
            .values
            .insert(field.name(), serde_json::Value::from(value))
        {
            if self.concat != "" {
                let orig = orig.as_str().unwrap_or("");
                let new = format!("{orig}{}{value}", self.concat);
                self.values
                    .insert(field.name(), serde_json::Value::from(new));
            }
        }
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
