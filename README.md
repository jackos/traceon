# Traceon - trace on json
An easy to use log and tracing formatter with a flattened json output, it only takes one line of code for good defaults, with an easy to use builder for configuration.

The main purpose of this crate is to simplify the concept of `tracing`, which is just adding context to log messages for better debugging and observability, this documentation will cover everything you need, [but the detailed tracing docs are here](https://docs.rs/tracing/latest/tracing/).

The only two crates you'll need in your `Cargo.toml` are:

```toml
[dependencies]
tracing = "0.1"
traceon = "0.1"
```
And you can write your first trace with:
```rust
fn main() {
    traceon::json();
    tracing::info!("a simple message");
}
```
Which will give the default output of (this is configurable):
```json
{
  "message": "a simple message",
  "level": 30,
  "time": "2022-12-27T10:16:24.570889Z",
  "file": "src/main.rs:14"
}
```
Or you can use the pretty defaults:
```rust
traceon::pretty();
```
```
22:22:03 INFO a simple message
    file:   examples/builder.rs:19
    module: builder
```
Or you can use the builder to create your own format:

```rust
use traceon::{TimeFormat, TimeZone, LevelFormat}
traceon::builder()
	// Source code filename and line number `src/main.rs::10`
    file(),
	// Target and module path `mybinary::mymodule::submodule`
    module(),
	// Concatenated span name where the event occured `parentspan::childspan`
    span(),

    time(TimeFormat::PrettyTime),
	// Change the casing of all the key names `camelCase` to `snake_case`
    case: Case,
    pretty: bool,
    concat: Option<String>,
    level: LevelFormat,
	// Put anything that implements write here to redirect output
	.writer(std::io::stderr());
```


Log levels are converted to numbers by default:
```text
trace: 10
debug: 20
info:  30
warn:  40
error: 50
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
    traceon::on();
    add(5, 10).await;
}
```

```json
{
  "message": "result: 15",
  "level": 30,
  "timestamp": "2022-12-29T02:53:42.6727Z",
  "module": "instrument",
  "file": "examples/instrument.rs:3",
  "span": "add",
  "a": 5,
  "b": 10
}
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
    traceon::on();
    let span = tracing::info_span!(
		"math_functions", 
		package_name = env!("CARGO_PKG_NAME",
	));
    add(5, 10).instrument(span).await;
}
```

```json
{
  "level": 30,
  "timestamp": "2022-12-29T03:03:14.450507Z",
  "module": "instrument_trait",
  "file": "examples/instrument_trait.rs:4",
  "message": "result: 15",
  "span": "math_functions",
  "package_name": "traceon"
}
```
The above `package_name` comes from the environment variable provided by cargo, which gets it from `Cargo.toml` at compile time and saves it for runtime:
```toml
[package]
name = "traceon"
```

__IMPORTANT!__ if you're calling an async functions with `.await`, only use the above two methods to create a span, [more details here](https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code) 

[examples/nested_spans.rs](examples/nested_spans.rs)
### Nested spans
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
    let span = tracing::info_span!("math_functions", package_name = env!("CARGO_PKG_NAME"));
    add(5, 10).instrument(span).await;
}
```

```json
{
  "level": 30,
  "time": "2022-12-28T12:19:43.386923Z",
  "file": "examples/nested_spans.rs:6",
  "message": "result: 15",
  "span": "math_functions::add",
  "a": 5,
  "package_name": "traceon",
  "b": 10
}
```
You can see above that the child span name `add` was concatenated to the parent span name `math_functions` with the characters `::`, if you prefer the span just overrides the parent you can turn this functionality off:
```rust
fn main() {
	traceon::builder().concat(None).on();
}
```
```json
{
  "span": "add",
}
```

or set it to something different:
```rust
fn main() {
	traceon::builder().concat(Some(">")).on();
}
```

```json
{
  "span": "math_functions>add"
}
```

The `add` function from above could be rewritten like this:

```rust
async fn add(a: i32, b: i32) {
    let _span = tracing::info_span!("add", a, b).entered();
    tracing::info!("result: {}", a + b);
}
```
This will cause the span to exit at the end of the function when _span is dropped, just remember to be very careful not to put any `.await` points when an `EnteredSpan` like `_span` above is being held.

### Turn off fields
This is an example of changing the default fields using the builder pattern, and takes the opportunity to introduce `tracing::event!`

[examples/builder.rs](examples/builder.rs)
```rust
use traceon::LevelFormat;
use tracing::Level;

fn main() {
    traceon::builder()
        .timestamp(false)
        .module(false)
        .span(false)
        .file(false)
        .level(LevelFormat::Lowercase)
        .on();

    tracing::info!("only this message and level as text");

    tracing::event!(
        Level::INFO,
        event_example = "add field and log it without a span"
    );

    let vector = vec![10, 15, 20];
    tracing::event!(
        Level::WARN,
        message = "add message field, and debug a vector",
        ?vector,
    );
}
```
```json
{
  "level": "info",
  "message": "only this message and level as text"
}
{
  "level": "info",
  "event_example": "add field and log it without a span"
}
{
  "level": "warn",
  "message": "add message field, and debug a vector",
  "vector": "[10, 15, 20]"
}
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
fn main() {
    let file_appender = tracing_appender::rolling::hourly("./", "test.log");
    traceon::builder().writer(file_appender).on();
    tracing::info!("wow cool!");
}
```
The writer accepts anything that implements the `Write` trait

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
Often you'll be consuming different crates that implement their own traces and you need all their keys to match a certain format, this example also demonstrates how to use different instances of `traceon` for a given scope with `on_thread()`, which returns a guard that will be dropped at the end of the scope, all current span fields and formatting will be dropped with it:
[examples/casing.rs](examples/casing.rs)
```rust
use traceon::Case;
use tracing::Level;
fn main() {
	let _guard = traceon::builder().case(Case::Pascal).on_thread();
	tracing::event!(
		Level::INFO,
		PascalCase = "test",
		camelCase = "test",
		snake_case = "test",
		SCREAMING_SNAKE_CASE = "test",
	);

	let _guard = traceon::builder().case(Case::Camel).on_thread();
	tracing::event!(
		Level::INFO,
		PascalCase = "test",
		camelCase = "test",
		snake_case = "test",
		SCREAMING_SNAKE_CASE = "test",
	);

	let _guard = traceon::builder().case(Case::Snake).on_thread();
	tracing::event!(
		Level::INFO,
		PascalCase = "test",
		camelCase = "test",
		snake_case = "test",
		SCREAMING_SNAKE_CASE = "test",
	);
}
```

<pre>
<div style="color: green;">2022-12-31T04:12:37.132Z INFO event triggered</div>
    CamelCase:          test
    PascalCase:         test
    ScreamingSnakeCase: test
    SnakeCase:          test

<div style="color: green;">2022-12-31T04:12:37.132Z INFO event triggered</div>
    camelCase:          test
    pascalCase:         test
    screamingSnakeCase: test
    snakeCase:          test

<div style="color: green;">2022-12-31T04:12:37.133Z INFO event triggered</div>
    camel_case:           test
    pascal_case:          test
    screaming_snake_case: test
    snake_case:           test
</pre>

```json
{
  "Level": 30,
  "Timestamp": "2022-12-29T03:51:55.640613Z",
  "Module": "casing",
  "File": "examples/casing.rs:6",
  "SnakeCase": "test",
  "PascalCase": "test",
  "CamelCase": "test",
  "ScreamingSnakeCase": "test"
}
{
  "level": 30,
  "timestamp": "2022-12-29T03:51:55.641014Z",
  "module": "casing",
  "file": "examples/casing.rs:16",
  "screamingSnakeCase": "test",
  "pascalCase": "test",
  "camelCase": "test",
  "snakeCase": "test"
}
{
  "level": 30,
  "timestamp": "2022-12-29T03:51:55.641204Z",
  "module": "casing",
  "file": "examples/casing.rs:27",
  "screaming_snake_case": "test",
  "camel_case": "test",
  "snake_case": "test",
  "pascal_case": "test"
}
```

## Pretty Printing Output
For pretty printing the output like the examples above, install [jq](https://stedolan.github.io/jq/download/) and run commands like:
```bash
cargo run | jq -R 'fromjson?'
```

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
