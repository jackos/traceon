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
