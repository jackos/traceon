use tracing::{info_span, Instrument};

#[tracing::instrument]
async fn level_1(param: &str) {
    let span = info_span!("wierd", level = 5);
    tracing::info!("level_1");
    level_2("level_1", param).instrument(span).await;
}
#[tracing::instrument]
async fn level_2(param: &str, param_2: &str) {
    tracing::info!("level_2");
    level_3("level_2", param, param).await;
}
#[tracing::instrument]
async fn level_3(param: &str, param_2: &str, param_3: &str) {
    level_4("level_3", param, param, param).await;
}
#[tracing::instrument]
async fn level_4(param: &str, param_2: &str, param_3: &str, param_4: &str) {
    tracing::info!("level_4");
    level_5("level_4", param, param, param, param).await;
}
#[tracing::instrument]
async fn level_5(param: &str, param_2: &str, param_3: &str, param_4: &str, param_5: &str) {
    tracing::info!("level_5");
}

#[tokio::main]
async fn main() {
    traceon::on();
    let _span = info_span!("coolness", message = "wow ok").entered();
    level_1("sick bra").await;
}
