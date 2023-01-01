use tracing::{info_span, Instrument};

#[tracing::instrument]
async fn level_1(param: &str) {
    tracing::info!("level_1");
    level_2("level_1", param).await;
}
#[tracing::instrument]
async fn level_2(param: &str, param_2: &str) {
    tracing::info!("level_2");
    level_3("level_2", param, param).await;
}
#[tracing::instrument]
async fn level_3(param: &str, param_2: &str, param_3: &str) {
    tracing::info!("level_3");
}

#[tokio::main]
async fn main() {
    traceon::builder().module().concat(Some("::")).on();
    let span = info_span!("base");
    level_1("base").instrument(span).await;
}
