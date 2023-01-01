use traceon::{SecondsFormat, TimeFormat, TimeZone};
fn main() {
    let _guard = traceon::builder().on_thread();
    tracing::info!("Default RFC3339 with zulu/utc time and milliseconds");
    let _guard = traceon::builder()
        .time(TimeFormat::PrettyTime)
        .timezone(TimeZone::Local)
        .on_thread();
    tracing::info!("Pretty and local time");
    let _guard = traceon::builder()
        .time(TimeFormat::RFC3339Options(SecondsFormat::Secs, false))
        .timezone(TimeZone::Local)
        .on_thread();
    tracing::info!("RFC3339 with timezone");
    let _guard = traceon::builder()
        .time(TimeFormat::EpochSeconds)
        .json()
        .on_thread();
    tracing::info!("number of seconds that have elapsed since 1970-01-01 00:00:00 utc");
}
