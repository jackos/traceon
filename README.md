# Traceon - trace on json
A simple log and trace formatter with a structured json output, it flattens events from nested spans, overriding parent fields.

The `tracing` crate is difficult to understand initially, this crate is designed to be as easy to use as possible with sensible defaults and configuration options. The only two crates you'll need in your `Cargo.toml` are:

```toml
[dependencies]
tracing = "0.1"
traceon = "0.1"
```

For pretty printing the output like the examples below, install [jq](https://stedolan.github.io/jq/download/) and run commands like:
```bash
cargo run | jq -R 'fromjson?'
```

By default `env-filter` is used at the `info` level, to change the level you can set an environment variable e.g. `RUST_LOG=warn`, all the options are [detailed here](https://docs.rs/env_logger/latest/env_logger/)

## Examples

### Simple Example
The extra fields outputted below are defaults that can be turned off:
```rust
fn main() {
    traceon::on();
    tracing::info!("a simple message");
}
```

```json
{
  "message": "a simple message",
  "level": 30,
  "time": "2022-12-27T10:16:24.570889Z",
  "file": "src/main.rs:14"
}
```

Log levels are converted to numbers by default:
```text
trace: 10
debug: 20
info:  30
warn:  40
error: 50
```

### \#\[instrument\] macro
If you're using normal functions or `async`, you can use the `tracing::instrument` macro to capture the parameters for each function call:
```rust
#[tracing::instrument]
async fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

#[tokio::main]
async fn main() {
    traceon::on();
    add(5, 10).await;
}
```

```json
{
  "message": "result: 15",
  "level": 30,
  "time": "2022-12-27T10:48:56.957671Z",
  "span": "add",
  "file": "src/main.rs:3",
  "a": 5,
  "b": 10
}
```

### Instrument trait
If you need to add some additional context to an async function, you can create a span and instrument it:
```rust
use tracing::Instrument;

async fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

#[tokio::main]
async fn main() {
    traceon::on();
    let span = tracing::info_span!("math functions", package_name = env!("CARGO_PKG_NAME"));
    add(5, 10).instrument(span).await;
}
```

```json
{
  "message": "result: 15",
  "level": 30,
  "time": "2022-12-27T11:11:25.540256Z",
  "span": "math functions",
  "file": "src/main.rs:4",
  "package_name": "testing_traceon"
}
```
The above `package_name` comes from the environment variable provided by cargo, which gets it from `Cargo.toml`:
```toml
[package]
name = "testing_traceon"
```

__IMPORTANT!__ for async functions only ever use the above two methods, which are the `#[instrument]` macro, and `Instrument` trait. The guard detailed below should not be used across async boundaries.

### Instrument trait and entered span
To combine the output from the two examples above we can enter a span with the arguments added to the trace:
```rust
use tracing::Instrument;

async fn add(a: i32, b: i32) {
    // Important! Don't put any `.await` calls in between `entered()` and `exit()`
    let span = tracing::info_span!("add", a, b).entered();
    tracing::info!("result: {}", a + b);
    span.exit();
}

#[tokio::main]
async fn main() {
    traceon::on();
    let span = tracing::info_span!("math functions", package_name = env!("CARGO_PKG_NAME"));
    add(5, 10).instrument(span).await;
}
```

```json
{
  "message": "result: 15",
  "level": 30,
  "time": "2022-12-27T11:18:46.805758Z",
  "span": "add",
  "file": "src/main.rs:5",
  "b": 10,
  "package_name": "testing_traceon",
  "a": 5
}
```
You can see above that the nested `"span": "add"` overrode the parent `"span": "math functions"`

The add function from above could be rewritten like this:

```rust
async fn add(a: i32, b: i32) {
    let _span = tracing::info_span!("add", a, b).entered();
    tracing::info!("result: {}", a + b);
}
```
This will cause the span to exit at the end of the function when _span is dropped, just remember to be very careful not to put any `.await` points when an `EnteredSpan` like `_span` above is being held.

### Turn off fields
This is an example of changing all the defaults fields to their opposites:

```rust
use traceon::LevelFormat;

mod helpers {
    pub fn trace() {
        tracing::info!("in helpers module");
    }
}

fn main() {
    traceon::builder()
        .module(true)
        .span(false)
        .file(false)
        .time(false)
        .level(LevelFormat::Off)
        .on();

    tracing::info!("only the module and message");
    helpers::trace();
}
```
```json

{
  "message": "only the module and message",
  "module": "bootstrap"
}
{
  "message": "in helpers module",
  "module": "bootstrap::helpers"
}
```
This was using a Cargo.toml with the binary renamed to `bootstrap` for demonstration purposes:

```toml
[[bin]]
name = "bootstrap"
path = "src/main.rs"
```

### Write to a file
If you wanted to write to log files instead of std, it's as simple adding the dependency to `Cargo.toml`:
```toml
[dependencies]
tracing-appender = "0.2.2"
```
And initializing it via the builder: 
```rust
fn main() {
    let file_appender = tracing_appender::rolling::hourly("./", "test.log");
    traceon::builder().writer(file_appender).on();
    tracing::info!("wow cool!");
}
```

### Compose with other layers
You can also use the formatting and storage layer with other tracing layers as you get more comfortable with the tracing ecosystem, e.g. to change the filter:

```rust
use traceon::Traceon;
use tracing_subscriber::{prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(Traceon::default())
        .with(EnvFilter::new("error"))
        .init();

    tracing::info!("info log message won't write to stdout");
    tracing::error!("only error messages will write to stdout");
}
```

## Performance
This crate uses the idea originated from: [LukeMathWalker/tracing-bunyan-formatter](https://github.com/LukeMathWalker/tracing-bunyan-formatter) of storing fields from visited spans in a `HashMap` instead of a `BTreeMap` which is more suited for flattening fields, and results in very similar performance to the json formatter in `tracing-subcriber`:

logging to a sink
![benchmark std sink](images/benchmark-std-sink.png)
units = nanosecond or billionth of a second

logging to stdout
![benchmark std out](images/benchmark-std-out.png)
units = microsecond or millionth of a second
