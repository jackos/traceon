use nu_ansi_term::{Color, Style};
// use erased_serde::{Serialize, Serializer};
use chrono::offset::TimeZone as TimeZoneTrait;
use chrono::{DateTime, Local, SecondsFormat, Utc};
use serde::ser::{SerializeMap, Serializer};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, Mutex},
};
use tracing::Level;
use tracing::{
    field::{Field, Visit},
    span::Attributes,
    Event, Id, Subscriber,
};
use tracing_subscriber::{
    layer::{Context, SubscriberExt},
    EnvFilter, Layer, Registry,
};

/// Private struct to initialize formatting and storage layers
/// All members can be modified through public methods.
#[derive(Clone)]
pub struct Traceon {
    json: bool,
    file: bool,
    module: bool,
    span_format: SpanFormat,
    case: Case,
    time: TimeFormat,
    timezone: TimeZone,
    join_fields: JoinFields,
    level: LevelFormat,
    writer: Arc<Mutex<dyn Write + Sync + Send>>,
    message_key: &'static str,
}

/// Change case of keys
#[derive(Clone)]
pub enum Case {
    /// Use original case for all keys
    None,
    /// Convert all keys to camelCase
    Camel,
    /// Convert all keys to PascalCase
    Pascal,
    /// Convert all keys to snake_case
    Snake,
}

/// Format the log level
#[derive(Copy, Clone)]
pub enum LevelFormat {
    /// Hide log levels
    None,
    /// Log level uppercase e.g. WARN
    Uppercase,
    /// Log level lowercase e.g. warn
    Lowercase,
    /// Log level as numbers
    ///     TRACE: 10
    ///     DEBUG: 20
    ///     INFO:  30
    ///     WARN:  40
    ///     ERROR: 50
    Number,
}

/// Format the span field
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpanFormat {
    /// Turn of span field
    None,
    /// Nested children spans join parent spans e.g. parent_span::child_span
    Join(&'static str),
    /// Nested children spans overwrite parent spans
    Overwrite,
}

impl Default for SpanFormat {
    fn default() -> Self {
        SpanFormat::Join("::")
    }
}

/// Join fields with characters
#[derive(Copy, Clone, Debug, Default)]
pub enum JoinFields {
    #[default]
    /// All nested span fields will overwrite parent span fields
    Overwrite,
    /// All nested span fields will join with parent spans e.g JoinFields::All("::")
    All(&'static str),
    /// Only declared nested span fields will join with parent spans e.g. JoinFields(Some("::", &["field_a", "field_b"]))
    Some(&'static str, &'static [&'static str]),
}

/// Change the time formatting
#[derive(Clone, PartialEq, Eq)]
pub enum TimeFormat {
    /// Turn off the time field
    None,
    /// Unix epoch seconds since 1970-01-01 00:00:00 e.g. 1672574869
    EpochSeconds,
    /// Unix epoch millseconds since 1970-01-01 00:00:00 e.g. 1672574869384
    EpochMilliseconds,
    /// Unix epoch microseconds since 1970-01-01 00:00:00 e.g. 1672574869384925
    EpochMicroseconds,
    /// Unix epoch nanoseconds since 1970-01-01 00:00:00 e.g. 1672575028752943000
    EpochNanoseconds,
    /// Well known format e.g. Sun, 01 Jan 2023 12:10:28 +0000
    RFC2822,
    /// Well known format like ISO 8601 e.g. 2022-12-31T00:15:08.241974+00:00
    RFC3339,
    /// Seconds format and bool to replace +00:00 timezone with Z e.g. (SecondsFormat::Secs, true) = 2022-12-31T00:15:08Z
    RFC3339Options(SecondsFormat, bool),
    /// Pretty Print the time in format HH:mm:SS
    PrettyTime,
    /// Pretty Print the date in format YYYY:MM::DD HH:mm:SS
    PrettyDateTime,
    /// Use a format string to change the datetime formate e.g. YYYY:MM::DD HH:mm:SS
    CustomFormat(&'static str),
}

/// Change the timezone
#[derive(Clone)]
pub enum TimeZone {
    /// Use +00:00 timezone
    UTC,
    /// Use local system time for the timezone
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

/// Convert a datetime to String based on the TimeFormat
pub fn time_convert<Tz: TimeZoneTrait>(now: DateTime<Tz>, time: &TimeFormat) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match time {
        TimeFormat::None => now.timestamp().to_string(),
        TimeFormat::EpochSeconds => now.timestamp().to_string(),
        TimeFormat::EpochMilliseconds => now.timestamp_millis().to_string(),
        TimeFormat::EpochMicroseconds => now.timestamp_micros().to_string(),
        TimeFormat::EpochNanoseconds => now.timestamp_nanos_opt().unwrap_or(0).to_string(),
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

/// Default values used for the builder
impl Default for Traceon {
    #[must_use]
    fn default() -> Traceon {
        Traceon {
            json: false,
            file: false,
            module: false,
            message_key: "message",
            span_format: SpanFormat::Join("::"),
            case: Case::None,
            time: TimeFormat::RFC3339,
            timezone: TimeZone::UTC,
            join_fields: JoinFields::Overwrite,
            level: crate::LevelFormat::Uppercase,
            writer: Arc::new(Mutex::new(std::io::stdout())),
        }
    }
}

impl Traceon {
    /// Turn the file field on:
    /// ```
    /// traceon::builder().file().on();
    /// ```
    ///
    /// pretty output:
    /// ```text
    ///     file: src/traceon.rs:68
    /// ```
    #[must_use]
    pub fn file(&mut self) -> &mut Self {
        self.file = true;
        self
    }

    /// Change formatting of span field, make children overrwrite the parent, or turn it off
    /// ```
    /// use traceon::SpanFormat;
    ///
    /// traceon::builder().span(SpanFormat::Join(">")).on();

    /// let _span = tracing::info_span!("level_1").entered();
    /// tracing::info!("span level 1");

    /// let _span = tracing::info_span!("level_2").entered();
    /// tracing::info!("span level 2");
    /// ```
    ///
    /// Pretty output:
    ///
    /// ```text
    /// 12:30:02 INFO span level 1
    ///     span: level_1
    ///
    /// 12:30:02 INFO span level 2
    ///     span: level_1>level_2
    /// ```
    ///
    /// To turn off concatenation of span fields:
    /// ```
    /// use traceon::SpanFormat;
    ///
    /// traceon::builder().span(SpanFormat::None);
    /// ```
    ///
    /// ```
    #[must_use]
    pub fn span(&mut self, span_format: SpanFormat) -> &mut Self {
        self.span_format = span_format;
        self
    }

    /// Change the key for the message field when using the json formatter
    /// ```
    /// traceon::builder().json().message_key("msg").on();
    /// traceon::info!("the message key is now msg");
    /// ```
    ///
    /// json output:
    /// ```json
    /// {
    ///     "msg": "the message key is now msg"
    /// }
    /// ```
    #[must_use]
    pub fn message_key(&mut self, message_key: &'static str) -> &mut Self {
        self.message_key = message_key;
        self
    }

    /// Turn module field on
    /// ```
    /// traceon::builder().module().on();
    /// ```
    ///
    /// pretty output:
    /// ```text
    ///     module: my_target::my_module
    /// ```
    #[must_use]
    pub fn module(&mut self) -> &mut Self {
        self.module = true;
        self
    }

    /**
    Choose to join (concatenate) values from the same field in nested spans:
    ```
    use traceon::JoinFields;
    traceon::builder()
        .join_fields(JoinFields::Some("||", &["field_b"]))
        .on();

    let _span_1 = tracing::info_span!("span_1", field_a = "original", field_b = "original").entered();
    let _span_2 = tracing::info_span!("span_2", field_a = "changed", field_b = "changed").entered();

    tracing::info!("testing field join");
    ```

    pretty output:
    ```text
    12:44:12 INFO testing field join
        field_a: changed
        field_b: original||changed
        span:    span_1::span_1
    ```
    */
    #[must_use]
    pub fn join_fields(&mut self, join_fields: JoinFields) -> &mut Self {
        self.join_fields = join_fields;
        self
    }
    /// Change time formatting
    #[must_use]
    pub fn time(&mut self, time_format: TimeFormat) -> &mut Self {
        self.time = time_format;
        self
    }
    /// Change time formatting
    #[must_use]
    pub fn level(&mut self, level_format: LevelFormat) -> &mut Self {
        self.level = level_format;
        self
    }
    /// Change timezone
    #[must_use]
    pub fn timezone(&mut self, timezone: TimeZone) -> &mut Self {
        self.timezone = timezone;
        self
    }
    /// Use json formatting instead of pretty formatting
    #[must_use]
    pub fn json(&mut self) -> &mut Self {
        self.json = true;
        self
    }
    /// Use any writer that is threadsafe and implements the `Write` trait
    #[must_use]
    pub fn writer(&mut self, writer: impl Write + Send + Sync + 'static) -> &mut Self {
        self.writer = Arc::new(Mutex::new(writer));
        self
    }
    /// Write to a buffer that you can share between threads by wrapping it in an Arc and Mutex
    #[must_use]
    pub fn buffer(&mut self, buffer: Arc<Mutex<impl Write + Send + Sync + 'static>>) -> &mut Self {
        self.writer = buffer;
        self
    }
    /// Change casing of keys to match a specefic format
    #[must_use]
    pub fn case(&mut self, case: Case) -> &mut Self {
        self.case = case;
        self
    }

    /// Turn on the storage, formatting and filter layers as a global default, which means all threads will inherit it but it can
    /// be overwritten for a single thread with for example: `let _guard = traceon::builder().on_thread();`
    ///
    /// # Panics
    ///
    /// Will panic if the global default subscriber is already set, use `try_on` instead to return a `Result`
    pub fn on(&self) {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(self.clone()).with(env_filter);

        // Panic if user is trying to set two global default subscribers
        tracing::subscriber::set_global_default(subscriber)
            .expect("more than one global default subscriber set");
    }

    /// Turn on the storage, formatting and filter layers as a global default, which means all threads will inherit it but it can
    /// be overwritten for a single thread with for example: `let _guard = traceon::builder().on_thread();`
    ///
    /// Returns a result which will be an error if the global default subscriber is already set
    pub fn try_on(&self) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(self.clone()).with(env_filter);

        tracing::subscriber::set_global_default(subscriber)
    }

    /**
    Turn on the storage, formatting and filter layers on the local thread returning a guard, when the guard is dropped the
    layers will be unsubscribed.

    # Examples

    ```
    // Turn on the subscriber with json formatting
    let _span = tracing::info_span!("the storage layer in this subscriber will have a field", field = "temp");
    let _guard = traceon::builder().json().on_thread();
    tracing::info!("first subscriber");


    // Drop the previous subscriber and storage for the fields, this new one has pretty formatting
    let _span = tracing::info!("the storage layer has been reset");
    let _guard = traceon::builder().on_thread();
    tracing::info!("second subscriber")
    ```

    Returns a result which will be an error if the global default subscriber is already set
    */
    pub fn on_thread(&self) -> tracing::subscriber::DefaultGuard {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = Registry::default().with(self.clone()).with(env_filter);

        tracing::subscriber::set_default(subscriber)
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
            Case::Pascal => ("Level", "File", "Module", "Time"),
            _ => ("level", "file", "module", "time"),
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
                let number = match *metadata.level() {
                    Level::TRACE => 10,
                    Level::DEBUG => 20,
                    Level::INFO => 30,
                    Level::WARN => 40,
                    Level::ERROR => 50,
                };

                if self.json {
                    map_serializer.serialize_entry(level_key, &number)?;
                } else {
                    write!(msg, "{} ", number)?;
                }
            }
            LevelFormat::None => (),
        }

        if !self.json {
            let style = match *event.metadata().level() {
                Level::TRACE => Style::new().fg(Color::Purple),
                Level::DEBUG => Style::new().fg(Color::Blue),
                Level::INFO => Style::new().fg(Color::Green),
                Level::WARN => Style::new().fg(Color::Yellow),
                Level::ERROR => Style::new().fg(Color::Red),
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
        for (mut key, value) in event_visitor.values.iter() {
            if self.json && key == &"message" {
                key = &self.message_key;
            }
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
        let mut event_visitor = JsonStorage::new(self.join_fields, self.span_format);
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
            if self.span_format != SpanFormat::None {
                if let Some(orig) = storage
                    .values
                    .insert(span_key, serde_json::Value::from(span.metadata().name()))
                {
                    match self.span_format {
                        SpanFormat::Overwrite => (),
                        SpanFormat::Join(concat) => {
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
                        SpanFormat::None => (),
                    }
                };
            }
            storage
        } else {
            let mut storage = JsonStorage::new(self.join_fields, self.span_format);
            if self.span_format != SpanFormat::None {
                storage
                    .values
                    .insert(span_key, serde_json::Value::from(span.metadata().name()));
            }
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
    pub join_fields: JoinFields,
    pub span_format: SpanFormat,
}

impl<'a> JsonStorage<'a> {
    pub fn new(join_fields: JoinFields, span_format: SpanFormat) -> Self {
        JsonStorage {
            values: HashMap::new(),
            join_fields,
            span_format,
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
        upper_or_underscore_last = ch.is_uppercase() || ch == '_';
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
            if field.name().to_ascii_lowercase() == "span" {
                if let SpanFormat::Join(chars) = self.span_format {
                    let orig = orig.as_str().unwrap_or("");
                    let new = format!("{orig}{chars}{value}");
                    self.values
                        .insert(field.name(), serde_json::Value::from(new));
                }
            } else {
                match self.join_fields {
                    JoinFields::Overwrite => (),
                    JoinFields::All(chars) => {
                        let orig = orig.as_str().unwrap_or("");
                        let new = format!("{orig}{chars}{value}");
                        self.values
                            .insert(field.name(), serde_json::Value::from(new));
                    }
                    JoinFields::Some(chars, fields) => {
                        if fields.contains(&field.to_string().as_str()) {
                            let orig = orig.as_str().unwrap_or("");
                            let new = format!("{orig}{chars}{value}");
                            self.values
                                .insert(field.name(), serde_json::Value::from(new));
                        }
                    }
                }
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
