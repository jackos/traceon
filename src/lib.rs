#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod traceon;
use crate::traceon::Traceon;
pub use crate::traceon::{Case, JoinFields, LevelFormat, SpanFormat, TimeFormat, TimeZone};
pub use chrono::SecondsFormat;
use tracing::subscriber::DefaultGuard;
pub use tracing::{
    debug, debug_span, error, error_span, event, info, info_span, instrument, trace, trace_span,
    warn, warn_span, Instrument, Level,
};

/** Returns a builder that can be configured before being turned on, or used as a layer for a subscriber.
```
use traceon::{Case, JoinFields, LevelFormat, SecondsFormat, SpanFormat, TimeFormat, Timezone, info};
traceon::builder()
    // Add field with source code filename and line number e.g. src/main.rs:10
    .file()
    // Add field with target and module path e.g. mybinary::mymodule::submodule
    .module()
    // Turn off field with joined span name where the event occured e.g. parentspan::childspan
    .span(SpanFormat::None)
    // If the time is recorded in local system timezone or UTC
    .timezone(Timezone::UTC)
    // Change the formatting of the time to RFC3339 with Seconds and Zulu
    .time(TimeFormat::RFC3339Options(SecondsFormat::Secs, true))
    // Change the casing of all the key names e.g. `camelCase` to `snake_case`
    .case(Case::Snake)
    // The characters used to concatenate field values that repeat in nested spans. Defaults to overwrite.
    .join_fields(JoinFields::All("::"))
    // Turn on json formatting instead of pretty output
    .json()
    // Change level value formatting to numbers for easier filtering
    // trace: 10
    // debug: 20
    // info:  30
    // warn:  40
    // error: 50
    .level(LevelFormat::Number)
    // Put anything that implements Write here to redirect output
    .writer(std::io::stderr())
    // This will activate it globally on all threads!
    .on();

info!("a simple message");
```
json output:
```json
{
    "timestamp": "2023-01-01T12:58:49Z",
    "level": 30,
    "module": "builder",
    "file": "examples/builder.rs:32",
    "message": "a simple message"
}
```
*/
pub fn builder() -> Traceon {
    Traceon::default()
}

/**
Turns on the pretty defaults which is local time with no date, where all the span fields are new indented lines, activating it globally on all threads.

# Panics
Will panic if a global default is already set

# Examples
```
traceon::json();
traceon::info!("a json message");
```
output prettified:
```json
{
  "time": "2023-01-02T04:46:12.715798+00:00",
  "level": "INFO",
  "message": "a json message"
}
```
*/
pub fn on() {
    Traceon::default()
        .timezone(TimeZone::Local)
        .time(TimeFormat::PrettyTime)
        .file()
        .module()
        .on();
}

/**
Turn on pretty defaults for the local thread returning a guard, when the guard is dropped the layers will be unsubscribed.

# Examples
Example of using two subscribers on the same thread, second event loses field from first span
```
use traceon::{info, info_span};

let _guard = traceon::on_thread();
let _span = info_span!("span_with_field", field = "temp", "cool").entered();
info!("first subscriber");

let _guard = traceon::json_thread();
let _span = info_span!("span_with_no_field").entered();
info!("second subscriber")
```

output:
```text
11:58:50 INFO first subscriber
    field: temp
    span:  span_with_field
```

```json
{
  "time": "2023-01-02T04:59:00.841691+00:00",
  "level": "INFO",
  "message": "second subscriber",
  "span": "span_with_no_field"
}
```
*/
pub fn on_thread() -> DefaultGuard {
    Traceon::default()
        .timezone(TimeZone::Local)
        .time(TimeFormat::PrettyTime)
        .file()
        .module()
        .on_thread()
}

/**
Turns on the json defaults which is one line of flattened json per event with RFC3339 UTC time, activating it globally on all threads.

# Panics
Will panic if a global default is already set

# Examples
```
traceon::json();
traceon::info!("a json message");
```
output prettified:
```json
{
  "time": "2023-01-02T04:46:12.715798+00:00",
  "level": "INFO",
  "message": "a json message"
}
```
*/
pub fn json() {
    Traceon::default()
        .file()
        .module()
        .timezone(TimeZone::UTC)
        .time(TimeFormat::RFC3339Options(SecondsFormat::Millis, true))
        .json()
        .on();
}

/**
Turn on json defaults for the local thread returning a guard, when the guard is dropped the layers will be unsubscribed.

# Examples
Example of using two subscribers on the same thread, second event loses field from first span
```
use traceon::{info, info_span};

let _guard = traceon::on_thread();
let _span = info_span!("span_with_field", field = "temp", "cool").entered();
info!("first subscriber");

let _guard = traceon::json_thread();
let _span = info_span!("span_with_no_field").entered();
info!("second subscriber")
```

output:
```text
11:58:50 INFO first subscriber
    field: temp
    span:  span_with_field

{
  "time": "2023-01-02T04:59:00.841691+00:00",
  "level": "INFO",
  "message": "second subscriber",
  "span": "span_with_no_field"
}
```
*/
pub fn json_thread() -> DefaultGuard {
    Traceon::default()
        .file()
        .module()
        .timezone(TimeZone::UTC)
        .time(TimeFormat::RFC3339Options(SecondsFormat::Millis, true))
        .json()
        .on_thread()
}
