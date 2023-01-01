use nu_ansi_term::{Color, Style};
// use erased_serde::{Serialize, Serializer};
use chrono::offset::TimeZone as TimeZoneTrait;
use chrono::SecondsFormat;
use chrono::{DateTime, Local, Utc};
use serde::ser::{SerializeMap, Serializer};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, Mutex},
};
use tracing::{field::Visit, span::Attributes, Event, Id, Subscriber};
use tracing_core::Field;
use tracing_log::AsLog;
use tracing_subscriber::{
    layer::{Context, SubscriberExt},
    EnvFilter, Layer, Registry,
};

#[derive(Clone)]
pub struct Traceon {
    writer: Arc<Mutex<dyn Write + Sync + Send>>,
    file: bool,
    module: bool,
    span: bool,
    time: TimeFormat,
    timezone: TimeZone,
    concat: Option<String>,
    level: LevelFormat,
    case: Case,
    json: bool,
}

#[derive(Clone)]
pub enum Case {
    None,
    Camel,
    Pascal,
    Snake,
}

#[derive(Copy, Clone)]
pub enum LevelFormat {
    None,
    Uppercase,
    Lowercase,
    Number,
}

#[derive(Clone, PartialEq, Eq)]
pub enum TimeFormat {
    None,
    EpochSeconds,
    EpochMilliseconds,
    EpochMicroseconds,
    EpochNanoseconds,
    /// Well known format e.g.
    RFC2822,
    /// Well known format similar to ISO 8601 e.g. 2022-12-31T00:15:08.241974+00:00
    RFC3339,
    /// Seconds format and bool to replace +00:00 timezone with Z e.g. (SecondsFormat::Secs, true) = 2022-12-31T00:15:08Z
    RFC3339Options(SecondsFormat, bool),
    /// Pretty Print the time in format HH:mm:SS
    PrettyTime,
    /// Pretty Print the date in format YYYY:MM::DD HH:mm:SS
    PrettyDateTime,
    CustomFormat(String),
}

#[derive(Clone)]
pub enum TimeZone {
    UTC,
    Local,
}

/// Convert json values with \n and \" characters to their escaped values when in pretty mode
pub fn clean_json_value(value: &Value) -> String {
    value
        .to_string()
        .trim_matches('"')
        .replace("\\\"", "\"")
        .replace("\\n", "\n    ")
}

pub fn time_convert<Tz: TimeZoneTrait>(now: DateTime<Tz>, time: &TimeFormat) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match time {
        TimeFormat::None => now.timestamp().to_string(),
        TimeFormat::EpochSeconds => now.timestamp().to_string(),
        TimeFormat::EpochMilliseconds => now.timestamp_millis().to_string(),
        TimeFormat::EpochMicroseconds => now.timestamp_micros().to_string(),
        TimeFormat::EpochNanoseconds => now.timestamp_subsec_nanos().to_string(),
        TimeFormat::RFC2822 => now.to_rfc2822(),
        TimeFormat::RFC3339 => now.to_rfc3339(),
        TimeFormat::RFC3339Options(seconds_format, use_z) => {
            now.to_rfc3339_opts(*seconds_format, *use_z)
        }
        TimeFormat::PrettyTime => now.format("%T").to_string(),
        TimeFormat::PrettyDateTime => now.format("%Y-%m-%d %T").to_string(),
        TimeFormat::CustomFormat(fmt) => now.format(fmt).to_string(),
    }
}

impl Default for Traceon {
    #[must_use]
    fn default() -> Traceon {
        Traceon {
            writer: Arc::new(Mutex::new(std::io::stdout())),
            concat: Some("::".into()),
            file: false,
            span: false,
            time: TimeFormat::RFC3339Options(SecondsFormat::Millis, true),
            timezone: TimeZone::UTC,
            module: false,
            json: false,
            case: Case::None,
            level: crate::LevelFormat::Uppercase,
        }
    }
}

impl Traceon {
    #[must_use]
    pub fn pretty_default() -> Self {
        Traceon {
            writer: Arc::new(Mutex::new(std::io::stdout())),
            concat: Some("::".into()),
            file: true,
            span: true,
            time: TimeFormat::PrettyTime,
            timezone: TimeZone::Local,
            module: true,
            json: true,
            case: Case::None,
            level: crate::LevelFormat::Uppercase,
        }
    }
    pub fn json_default() -> Self {
        Traceon {
            writer: Arc::new(Mutex::new(std::io::stdout())),
            concat: Some("::".into()),
            file: true,
            span: true,
            time: TimeFormat::RFC3339,
            timezone: TimeZone::UTC,
            module: true,
            json: false,
            case: Case::None,
            level: crate::LevelFormat::Uppercase,
        }
    }
    /// Turn the file field on or off
    /// ```
    /// traceon::json().file(true).on();
    /// tracing::info!("file field on");
    /// ```
    ///
    /// ```json
    /// {
    ///     "message": "file field on",
    ///     "file": "src/traceon.rs:68"
    /// }
    /// ```
    #[must_use]
    pub fn file(&mut self) -> &mut Self {
        self.file = true;
        self
    }

    /// Turn span fields on or off
    /// ```
    /// traceon::json().span(true).on();
    /// let _span = tracing::info_span!("level_1").entered();
    /// tracing::info!("span level 1");
    ///
    /// let _span = tracing::info_span!("level_2").entered();
    /// tracing::info!("span level 2");
    /// ```
    ///
    /// output:
    ///
    /// ```json
    /// {
    ///        "message": "span field is on",
    ///        "span": "level_1"
    ///    }
    /// ```
    /// ```json
    ///    {
    ///        "message": "span field is on",
    ///        "span": "level_1::level_2"
    ///    }
    /// ```
    ///
    /// To turn off concatenation of span fields:
    ///
    /// ```
    /// traceon::json().concat(None).on();
    /// let _span = tracing::info_span!("level_1").entered();
    /// tracing::info!("span field is on");
    ///
    /// let _span = tracing::info_span!("level_2").entered();
    /// tracing::info!("span field is on");
    /// ```
    ///
    /// output:
    ///
    /// ```json
    /// {
    ///        "message": "span field is on",
    ///        "span": "level_1"
    ///    }
    /// ```
    /// ```json
    ///    {
    ///        "message": "span field is on",
    ///        "span": "level_2"
    ///    }
    /// ```
    ///
    #[must_use]
    pub fn span(&mut self) -> &mut Self {
        self.span = true;
        self
    }

    /// Turn module on or off
    /// ```
    /// traceon::builder().module().writer(std::io::stderr()).on();
    /// tracing::info!("short message");
    /// ```
    #[must_use]
    pub fn module(&mut self) -> &mut Self {
        self.module = true;
        self
    }
    #[must_use]
    pub fn concat(&mut self, concat: Option<&str>) -> &mut Self {
        if let Some(concat) = concat {
            self.concat = Some(concat.to_string());
        } else {
            self.concat = None;
        }
        self
    }
    #[must_use]
    pub fn time(&mut self, time_format: TimeFormat) -> &mut Self {
        self.time = time_format;
        self
    }
    #[must_use]
    pub fn level(&mut self, level_format: LevelFormat) -> &mut Self {
        self.level = level_format;
        self
    }
    #[must_use]
    pub fn timezone(&mut self, timezone: TimeZone) -> &mut Self {
        self.timezone = timezone;
        self
    }
    #[must_use]
    pub fn json(&mut self) -> &mut Self {
        self.json = true;
        self
    }
    #[must_use]
    pub fn writer(&mut self, writer: impl Write + Send + Sync + 'static) -> &mut Self {
        self.writer = Arc::new(Mutex::new(writer));
        self
    }
    #[must_use]
    pub fn case(&mut self, case: Case) -> &mut Self {
        self.case = case;
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

    pub fn format_field(
        &self,
        key: &str,
        value: &str,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.json && key.to_ascii_lowercase() != "message" {
            writeln!(buffer, "{}: {}", key, value)?;
        };
        Ok(())
    }
    /// Serialize a single event
    fn serialize<S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>(
        &self,
        event: &Event<'_>,
        ctx: Context<'_, S>,
        event_visitor: &mut JsonStorage,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut msg = Vec::new();
        let mut pretty_buffer = Vec::new();
        let mut json_buffer = Vec::new();

        let mut serializer = serde_json::Serializer::new(&mut json_buffer);
        let mut map_serializer = serializer.serialize_map(None)?;
        let current_span = ctx.lookup_current();
        event.record(event_visitor);

        let (level_key, file_key, module_key, timestamp_key) = match self.case {
            Case::Pascal => ("Level", "File", "Module", "Timestamp"),
            _ => ("level", "file", "module", "timestamp"),
        };

        let metadata = event.metadata();

        if self.time != TimeFormat::None {
            let time_string = match self.timezone {
                TimeZone::UTC => {
                    let now = Utc::now();
                    time_convert(now, &self.time)
                }
                TimeZone::Local => {
                    let now = Local::now();
                    time_convert(now, &self.time)
                }
            };
            if self.json {
                map_serializer.serialize_entry(timestamp_key, &time_string)?;
            } else {
                write!(msg, "{time_string} ")?;
            }
        }
        match self.level {
            LevelFormat::Uppercase => {
                if self.json {
                    map_serializer.serialize_entry(level_key, &metadata.level().to_string())?;
                } else {
                    write!(msg, "{} ", metadata.level())?;
                }
            }
            LevelFormat::Lowercase => {
                if self.json {
                    map_serializer.serialize_entry(
                        level_key,
                        &metadata.level().to_string().to_ascii_lowercase(),
                    )?;
                } else {
                    write!(
                        msg,
                        "{} ",
                        metadata.level().to_string().to_ascii_lowercase()
                    )?;
                }
            }
            LevelFormat::Number => {
                let number = match metadata.level().as_log() {
                    log::Level::Error => 50u16,
                    log::Level::Warn => 40,
                    log::Level::Info => 30,
                    log::Level::Debug => 20,
                    log::Level::Trace => 10,
                };

                if self.json {
                    map_serializer.serialize_entry(level_key, &number)?;
                } else {
                    write!(msg, "{} ", number)?;
                }
            }
            LevelFormat::None => (),
        }
        // let x = d.format(&format).expect("Failed to format the time");

        if !self.json {
            let style = match event.metadata().level().as_log() {
                log::Level::Trace => Style::new().fg(Color::Purple),
                log::Level::Debug => Style::new().fg(Color::Blue),
                log::Level::Info => Style::new().fg(Color::Green),
                log::Level::Warn => Style::new().fg(Color::Yellow),
                log::Level::Error => Style::new().fg(Color::Red),
            };

            if let Some(value) = event_visitor.values.get("message") {
                let message = clean_json_value(value);
                write!(msg, "{message}")?;
            } else {
                write!(msg, "event triggered")?;
            };
            let msg = String::from_utf8_lossy(&msg);
            let msg = msg.trim();
            writeln!(pretty_buffer, "{}", style.paint(msg))?;
        }

        let mut fields = Vec::new();

        if self.module {
            if self.json {
                map_serializer
                    .serialize_entry(module_key, metadata.module_path().unwrap_or_default())?;
            } else {
                fields.push((
                    module_key.to_string(),
                    metadata.module_path().unwrap_or_default().to_string(),
                ));
            }
        }

        if self.file {
            let value = format!(
                "{}:{}",
                metadata.file().unwrap_or_default(),
                metadata.line().unwrap_or_default()
            );

            if self.json {
                map_serializer.serialize_entry(file_key, &value)?;
            } else {
                fields.push((file_key.to_string(), value.to_string()));
            }
        }

        // Add all the fields from the current event.
        for (key, value) in event_visitor.values.iter() {
            let key = match self.case {
                Case::Snake => snake(key),
                Case::Pascal => pascal(key),
                Case::Camel => camel(key),
                Case::None => key.to_string(),
            };

            if self.json {
                map_serializer.serialize_entry(&key, value)?;
            } else if key.to_ascii_lowercase() != "message" {
                fields.push((key.to_string(), clean_json_value(value)));
            }
        }

        // Add all the fields from the current span, if we have one.
        if let Some(span) = &current_span {
            let extensions = span.extensions();
            if let Some(visitor) = extensions.get::<JsonStorage>() {
                for (key, value) in &visitor.values {
                    let key = match self.case {
                        Case::Snake => snake(key),
                        Case::Pascal => pascal(key),
                        Case::Camel => camel(key),
                        Case::None => key.to_string(),
                    };

                    if self.json {
                        map_serializer.serialize_entry(&key, value)?;
                    } else if key.to_ascii_lowercase() != "message" {
                        fields.push((key.to_string(), clean_json_value(value)));
                    }
                }
            }
        }
        if !self.json {
            fields.sort_by(|a, b| a.0.cmp(&b.0));
            let mut max_len = 0;
            for field in &fields {
                if field.0.len() > max_len {
                    max_len = field.0.len();
                }
            }
            for field in fields {
                let mut seperator = ": ".to_string();
                let spaces = max_len - field.0.len();
                for _ in 0..spaces {
                    seperator += " ";
                }
                writeln!(pretty_buffer, "    {}{}{}", field.0, seperator, field.1)?;
            }
        }
        map_serializer.end()?;
        if self.json {
            Ok(json_buffer)
        } else {
            Ok(pretty_buffer)
        }
    }
}

impl<S> Layer<S> for Traceon
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut event_visitor = JsonStorage::new(self.concat.clone());
        match self.serialize(event, ctx, &mut event_visitor) {
            Ok(mut buffer) => {
                buffer.write_all(b"\n").unwrap();
                self.writer.lock().unwrap().write_all(&buffer).unwrap();
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }

    /// This is the only occasion we have to store the fields attached to the span
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let span_key = match self.case {
            Case::Pascal => "Span",
            _ => "span",
        };
        // We want to inherit the fields from the parent span, if there is one.
        let mut visitor = if let Some(parent_span) = span.parent() {
            let mut extensions = parent_span.extensions_mut();
            let mut storage = extensions
                .get_mut::<JsonStorage>()
                .map(|v| v.to_owned())
                .unwrap_or_default();
            if let Some(orig) = storage
                .values
                .insert(span_key, serde_json::Value::from(span.metadata().name()))
            {
                if let Some(concat) = &self.concat {
                    storage.values.insert(
                        span_key,
                        serde_json::Value::from(format!(
                            "{}{}{}",
                            orig.as_str().unwrap_or(""),
                            concat,
                            span.metadata().name()
                        )),
                    );
                }
            };
            storage
        } else {
            let mut storage = JsonStorage::new(self.concat.clone());
            storage
                .values
                .insert(span_key, serde_json::Value::from(span.metadata().name()));
            storage
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
    pub concat: Option<String>,
}

impl<'a> JsonStorage<'a> {
    pub fn new(concat: Option<String>) -> Self {
        JsonStorage {
            values: HashMap::new(),
            concat,
        }
    }
}

fn snake(key: &str) -> String {
    let mut snake = String::new();
    let mut upper_or_underscore_last = false;
    for (i, ch) in key.char_indices() {
        if i > 0 && ch.is_uppercase() && !upper_or_underscore_last {
            snake.push('_');
        }
        if ch.is_uppercase() || ch == '_' {
            upper_or_underscore_last = true;
        } else {
            upper_or_underscore_last = false;
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

fn pascal(key: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    let mut upper_last = false;
    for ch in key.chars() {
        if ch.is_lowercase() {
            upper_last = false;
        }
        if ch == '_' {
            capitalize = true;
            upper_last = false;
        } else if upper_last {
            pascal.push(ch.to_ascii_lowercase());
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
            upper_last = true;
        } else {
            pascal.push(ch);
            upper_last = false;
        }
    }
    pascal
}

fn camel(key: &str) -> String {
    let pascal = pascal(key);
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
            // If self.concat is Some(_), instead of replacing value concatenate it
            if let Some(concat) = &self.concat {
                let orig = orig.as_str().unwrap_or("");
                let new = format!("{orig}{}{value}", concat);
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
