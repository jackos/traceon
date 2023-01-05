use traceon::{event, Case, Level};
fn main() {
    let _guard = traceon::builder().case(Case::Pascal).on_thread();
    event!(
        Level::INFO,
        message = "PascalCase",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
    let _guard = traceon::builder().case(Case::Camel).on_thread();
    event!(
        Level::INFO,
        message = "camelCase",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
    let _guard = traceon::builder().case(Case::Snake).on_thread();

    event!(
        Level::INFO,
        message = "snake_case",
        PascalCase = "test",
        camelCase = "test",
        snake_case = "test",
        SCREAMING_SNAKE_CASE = "test",
    );
}
