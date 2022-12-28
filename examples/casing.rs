fn main() {
    traceon::on();
    let _span = tracing::info_span!("wow", BadCase = "change the key to snake case").entered();
    tracing::info!("make sure PascalCase changes to snake_case");
}
