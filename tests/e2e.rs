use claims::assert_some_eq;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{info, span, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

/// Tests have to be run on a single thread because we are re-using the same buffer for
/// all of them.
type InMemoryBuffer = Mutex<Vec<u8>>;
static BUFFER: Lazy<InMemoryBuffer> = Lazy::new(|| Mutex::new(vec![]));

// Run a closure and collect the output emitted by the tracing instrumentation using an in-memory buffer.
fn run_and_get_raw_output<F: Fn()>(action: F) -> String {
    let mut default_fields = HashMap::new();
    default_fields.insert("custom_field".to_string(), json!("custom_value"));
    let traceon = traceon::builder();
    let subscriber = Registry::default().with(traceon);
    tracing::subscriber::with_default(subscriber, action);

    // Return the formatted output as a string to make assertions against
    let mut buffer = BUFFER.lock().unwrap();
    let output = buffer.to_vec();
    // Clean the buffer to avoid cross-tests interactions
    buffer.clear();
    String::from_utf8(output).unwrap()
}

// Run a closure and collect the output emitted by the tracing instrumentation using
// an in-memory buffer as structured new-line-delimited JSON.
fn run_and_get_output<F: Fn()>(action: F) -> Vec<Value> {
    run_and_get_raw_output(action)
        .lines()
        .filter(|&l| !l.is_empty())
        .inspect(|l| println!("{}", l))
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect()
}

// Instrumented code to be run to test the behaviour of the tracing instrumentation.
fn test_action() {
    let a = 2;
    let span = span!(Level::DEBUG, "shaving_yaks", a);
    let _enter = span.enter();

    info!("pre-shaving yaks");
    let b = 3;
    let new_span = span!(Level::DEBUG, "inner shaving", b);
    let _enter2 = new_span.enter();

    info!("shaving yaks");
}

#[test]
fn each_line_is_valid_json() {
    let tracing_output = run_and_get_raw_output(test_action);

    // Each line is valid JSON
    for line in tracing_output.lines().filter(|&l| !l.is_empty()) {
        assert!(serde_json::from_str::<Value>(line).is_ok());
    }
}

#[test]
fn each_line_has_the_mandatory_fields() {
    let tracing_output = run_and_get_output(test_action);

    for record in tracing_output {
        assert!(record.get("span").is_some());
        assert!(record.get("level").is_some());
        assert!(record.get("time").is_some());
        assert!(record.get("message").is_some());
    }
}

#[test]
fn encode_f64_as_numbers() {
    let f64_value: f64 = 0.5;
    let action = || {
        let span = span!(
            Level::DEBUG,
            "parent_span_f64",
            f64_field = tracing::field::Empty
        );
        let _enter = span.enter();
        span.record("f64_field", f64_value);
        info!("testing f64");
    };
    let tracing_output = run_and_get_output(action);

    for record in tracing_output {
        if record
            .get("msg")
            .and_then(Value::as_str)
            .map_or(false, |msg| msg.contains("testing f64"))
        {
            let observed_value = record.get("f64_field").and_then(|v| v.as_f64());
            assert_some_eq!(observed_value, f64_value);
        }
    }
}

#[test]
fn elapsed_milliseconds_are_present_on_exit_span() {
    let tracing_output = run_and_get_output(test_action);

    for record in tracing_output {
        if record
            .get("msg")
            .and_then(Value::as_str)
            .map_or(false, |msg| msg.ends_with("END]"))
        {
            assert!(record.get("elapsed_milliseconds").is_some());
        }
    }
}
