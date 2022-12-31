use traceon::Case;
use tracing::Level;
fn main() {
    {
        let _guard = traceon::builder().pretty().case(Case::Pascal).on_thread();
        tracing::event!(
            Level::INFO,
            PascalCase = "test",
            camelCase = "test",
            snake_case = "test",
            SCREAMING_SNAKE_CASE = "test",
        );
    }
    {
        let _guard = traceon::builder().pretty().case(Case::Camel).on_thread();
        tracing::event!(
            Level::INFO,
            PascalCase = "test",
            camelCase = "test",
            snake_case = "test",
            SCREAMING_SNAKE_CASE = "test",
        );
    }
    {
        let _guard = traceon::builder().pretty().case(Case::Snake).on_thread();

        tracing::event!(
            Level::INFO,
            PascalCase = "test",
            camelCase = "test",
            snake_case = "test",
            SCREAMING_SNAKE_CASE = "test",
        );
    }
}
