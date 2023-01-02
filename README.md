# Traceon
_Pronounced `trace on`_

An easy to use log and tracing formatter with a flattened json or pretty output.

It simplifies `tracing`, which adds context to log messages for better observability especially in async or multithreaded contexts, this documentation will get you up and running [but the detailed tracing docs are here](https://docs.rs/tracing/latest/tracing/).

The only two crates you need in `Cargo.toml` are:

```toml
[dependencies]
tracing = "0.1"
traceon = "0.2"
```

You can write your first human readable trace with:
```rust
use tracing::{info, instrument};

#[instrument]
fn add(a: i32, b: i32) {
    info!("result: {}", a + b);
}

fn main() {
    traceon::builder().on();
    add(5, 10);
}

```
Which will give the default output:
![images/pretty.svg](images/pretty.svg)

To turn the output into json:
```rust
traceon::builder().json().on();
```
```json
{
  "time": "2023-01-01T05:28:13.447Z",
  "level": "INFO",
  "message": "result: 15",
  "a": 5,
  "b": 10,
  "span": "add"
}
```
This example shows all the options you can change with the builder:
```rust
use traceon::{Case, JoinFields, LevelFormat, SecondsFormat, SpanFormat, TimeFormat, TimeZone};
fn main() {
    traceon::builder()
        // Add field with source code filename and line number e.g. src/main.rs:10
        .file()
        // Add field with target and module path e.g. mybinary::mymodule::submodule
        .module()
        // Turn off field with joined span name where the event occured e.g. parentspan::childspan
        .span(SpanFormat::None)
        // If the time is recorded in local system timezone or UTC
        .timezone(TimeZone::UTC)
        // Change the formatting of the time to RFC3339 with Seconds and Zulu
        .time(TimeFormat::RFC3339Options(SecondsFormat::Secs, true))
        // Change the casing of all the key names e.g. `camelCase` to `snake_case`
        .case(Case::Snake)
        // The characters used to concatenate field values that repeat in nested spans. Defaults to ::
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

    tracing::info!("a simple message");
}

```
Output
```json
{
	"timestamp": "2023-01-01T03:26:48Z",
	"level": 30,
	"module": "builder",
	"file": "examples/builder.rs:27",
	"message": "a simple message"
}
```
The defaults of the builder are optimized for local development, that is pretty output with just the local time, when moving to production e.g. on a cloud provider, you're likely going to want to turn the time off as they often record `stdout` and capture their own timestamps:
```rust
use traceon::TimeFormat;

traceon::builder().time(TimeFormat::None).json();
```

`env-filter` is used by default at the `info` level to filter any messages out at the `debug` or `trace` level, to change the level you can set an environment variable e.g. `RUST_LOG=warn` which would filter out `info` level as well, all the options are [detailed here](https://docs.rs/env_logger/latest/env_logger/)

## Examples

### \#\[instrument\] macro
You can use the `tracing::instrument` macro with both `async` and normal functions to capture the arguments used in each function call:

[examples/instrument.rs](examples/instrument.rs)
```rust
#[tracing::instrument]
async fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

#[tokio::main]
async fn main() {
    traceon::builder().on();
    add(5, 10).await;
}
```

```text
06:16:30 INFO result: 15
    a:    5
    b:    10
    span: add
```

### Instrument trait
If you need to add additional context to an async function, you can create a span and instrument it:

[examples/instrument.rs](examples/instrument_trait.rs)
```rust
use tracing::Instrument;

async fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

#[tokio::main]
async fn main() {
    traceon::builder().on();
    let span = tracing::info_span!("math_functions", package_name = env!("CARGO_PKG_NAME",));
    add(5, 10).instrument(span).await;
}
```

```text
06:18:55 INFO result: 15
    package_name: traceon
    span:         math_functions
```
The above `package_name` comes from the environment variable provided by cargo, which gets it from `Cargo.toml` at compile time and saves it for runtime:
```toml
[package]
name = "traceon"
```

[examples/nested_spans.rs](examples/nested_spans.rs)
### Entered spans
This creates a span and returns a guard, as long as that guard is in scope the span will be active:
```rust
fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

fn main() {
    traceon::builder().on();
    let _guard = tracing::info_span!("math", package_name = env!("CARGO_PKG_NAME")).entered();
    add(5, 10);
}
```

```text
06:26:21 INFO result: 15
    package_name: traceon
    span:         math
```

> **Warning**
> If `add()` was an `async fn`, holding onto the guard would cause memory leaks and information loss, you must use the above `instrument` macro or trait instead, [more details here](https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code) 

> **Note**
> Just remember don't call `.await` while holding onto a guard


### Nested spans
By default the span name will be joined with the characters `::` for nested spans:
```rust
#[tracing::instrument]
fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

fn main() {
    traceon::builder().on();
    let _guard = tracing::info_span!("math", package_name = env!("CARGO_PKG_NAME")).entered();
    add(5, 10);
}
```
```text
06:33:57 INFO result: 15
    a:            5
    b:            10
    package_name: traceon
    span:         math::add
```

You can set this to overwrite if you prefer:
```rust
use traceon::SpanFormat;
traceon::builder().span(SpanFormat::Overwrite).on();
```

```text
06:36:00 INFO result: 15
    a:            5
    b:            10
    package_name: traceon
    span:         add
```

By default all the other fields overwrite if a nested span has the same field name, you can join all fields or specific fields if you prefer (this only effects text values):

```rust
use traceon::JoinFields;
traceon::builder()
	.join_fields(JoinFields::Some("||", &["field_b"]))
	.on();

let _span_1 = tracing::info_span!("span_1", field_a = "original", field_b = "original").entered();
let _span_2 = tracing::info_span!("span_2", field_a = "changed", field_b = "changed").entered();

tracing::info!("testing field join");
```

output:
```text
12:44:12 INFO testing field join
	field_a: changed
	field_b: original||changed
	span:    span_1::span_1
```


### Event
`tracing::event!` allows you to add fields to message without having to create a span, just remember to put the level e.g. `tracing::Level::INFO` as the first parameter, this also shows how to create a custom message in an event, and how to output a `Debug` implementation:

[examples/builder.rs](examples/builder.rs)
```rust
use tracing::Level;

fn main() {
    traceon::builder().on();

    tracing::event!(
        Level::INFO,
        event_example = "add field and log it without a span"
    );

    let vector = vec![10, 15, 20];
    tracing::event!(
        Level::WARN,
        message = "overwrite message, and debug a vector",
        ?vector,
    );
}
```
```text
06:47:00 INFO event triggered
    event_example: add field and log it without a span

06:47:00 WARN overwrite message, and debug a vector
    vector: [10, 15, 20]
```

### Write to a file
If you wanted to write to log files instead of std, it's as simple adding the dependency to `Cargo.toml`:
```toml
[dependencies]
tracing-appender = "0.2.2"
```
And initializing it via the builder: 

[examples/file_writer.rs](examples/file_writer.rs)
```rust
let file_appender = tracing_appender::rolling::hourly("./", "test.log");
traceon::builder().json().writer(file_appender).on();
tracing::info!("wow cool!");
```
The writer accepts anything that implements the `Write` trait, if you want to hold onto a buffer wrapped in an `Arc` and `Mutex` there is `buffer()` method on the builder.

### Compose with other layers
You can also use the formatting layer with other tracing layers as you get more comfortable with the tracing ecosystem, for example to change the filter:

[examples/compose.rs](examples/compose.rs)
```rust
use tracing_subscriber::{prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(traceon::builder())
        .with(EnvFilter::new("error"))
        .init();

    tracing::info!("info log message won't write to stdout");
    tracing::error!("only error messages will write to stdout");
}
```

### Change the case of keys
Often you'll be consuming different crates that implement their own traces and you need all their keys to match a certain format, this example also demonstrates how to use different instances of `traceon` for a given scope with `on_thread()`, which returns a guard so the subscriber is only running on the current thread, and will be turned off when the guard is dropped.

[examples/casing.rs](examples/casing.rs)
```rust
use traceon::Case;
use tracing::Level;
fn main() {
    let _guard = traceon::builder().case(Case::Pascal).on_thread();
    tracing::event!(
        Level::INFO,
        message = "PascalCase",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
    let _guard = traceon::builder().case(Case::Camel).on_thread();
    tracing::event!(
        Level::INFO,
        message = "camelCase",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
    let _guard = traceon::builder().case(Case::Snake).on_thread();

    tracing::event!(
        Level::INFO,
        message = "snake_case",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
}
```
Output:
![images/casing.svg](images/casing.svg)

## Performance
This crate uses the idea originated from: [LukeMathWalker/tracing-bunyan-formatter](https://github.com/LukeMathWalker/tracing-bunyan-formatter) of storing fields from visited spans in a `HashMap` instead of a `BTreeMap` which is more suited for flattening fields, and results in very similar performance to the json formatter in `tracing-subcriber`:

### logging to a sink
![benchmark std sink](images/benchmark-std-sink.png)
units = nanosecond or billionth of a second

### logging to stdout
![benchmark std out](images/benchmark-std-out.png)
units = microsecond or millionth of a second

### Nested spans three levels deep with concatenated fields:
![benchmark std out](images/benchmark-async.png)
