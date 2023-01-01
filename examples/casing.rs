use traceon::Case;
use tracing::Level;
fn main() {
    let _guard = traceon::builder().json().case(Case::Pascal).on_thread();
    tracing::event!(
        Level::INFO,
        message = "PascalCase",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
    let _guard = traceon::builder().json().case(Case::Camel).on_thread();
    tracing::event!(
        Level::INFO,
        message = "camelCase",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
    let _guard = traceon::builder().json().case(Case::Snake).on_thread();

    tracing::event!(
        Level::INFO,
        message = "snake_case",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
}
