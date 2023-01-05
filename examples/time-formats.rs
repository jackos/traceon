use traceon::{SecondsFormat, TimeFormat, Timezone};
fn main() {
    let _guard = traceon::builder().on_thread();
    tracing::info!("Default RFC3339 with zulu/utc time and milliseconds");

    let _guard = traceon::builder()
        .time(TimeFormat::PrettyTime)
        .timezone(Timezone::Local)
        .on_thread();
    tracing::info!("Pretty and local time");

    let _guard = traceon::builder()
        .time(TimeFormat::PrettyDateTime)
        .on_thread();
    tracing::info!("PrettyDateTime");

    let _guard = traceon::builder()
        .time(TimeFormat::RFC3339Options(SecondsFormat::Secs, false))
        .timezone(Timezone::Local)
        .on_thread();
    tracing::info!("RFC3339 with timezone");

    let _guard = traceon::builder()
        .time(TimeFormat::EpochSeconds)
        .on_thread();
    tracing::info!("Epoch seconds");

    let _guard = traceon::builder()
        .time(TimeFormat::EpochMilliseconds)
        .on_thread();
    tracing::info!("Epoch milliseconds");

    let _guard = traceon::builder()
        .time(TimeFormat::EpochMicroseconds)
        .on_thread();
    tracing::info!("Epoch microseconds");

    let _guard = traceon::builder()
        .time(TimeFormat::EpochNanoseconds)
        .on_thread();
    tracing::info!("Epoch Nanoseconds");

    let _guard = traceon::builder()
        .time(TimeFormat::CustomFormat("%Y-%m-%d"))
        .on_thread();
    tracing::info!("custom format %Y-%m-%d");

    let _guard = traceon::builder().time(TimeFormat::RFC2822).on_thread();
    tracing::info!("RFC2822");
}
