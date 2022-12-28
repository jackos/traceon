use std::collections::BTreeMap;
use tracing_subscriber::Layer;
pub struct CustomLayer {
    pub file: bool,
}

impl CustomLayer {
    pub fn new() -> Self {
        Self { file: true }
    }
}

#[derive(Debug)]
struct CustomFieldStorage(BTreeMap<String, serde_json::Value>);

impl<S> Layer<S> for CustomLayer
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut spans = vec![];
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let extensions = span.extensions();
                let storage = extensions.get::<CustomFieldStorage>().unwrap();
                let field_data: &BTreeMap<String, serde_json::Value> = &storage.0;
                spans.push(serde_json::json!({
                    "target": span.metadata().target(),
                    "name": span.name(),
                    "level": span.metadata().level().to_string(),
                    "fields": field_data,
                }));
            }
        }
        // The fields of the event
        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        event.record(&mut visitor);

        // And create our output
        let output = serde_json::json!({
            "target": event.metadata().target(),
            "name": event.metadata().name(),
            "level": event.metadata().level().to_string(),
            "fields": fields,
            "spans": spans,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    fn on_record(
        &self,
        span: &tracing_core::span::Id,
        values: &tracing_core::span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Get the span whose data is being recorded
        let span = ctx.span(span).unwrap();

        // Get a mutable reference to the data we created in new_span
        let mut extensions_mut = span.extensions_mut();
        let custom_field_storage: &mut CustomFieldStorage =
            extensions_mut.get_mut::<CustomFieldStorage>().unwrap();
        let json_data: &mut BTreeMap<String, serde_json::Value> = &mut custom_field_storage.0;

        // And add to using our old friend the visitor!
        let mut visitor = JsonVisitor(json_data);
        values.record(&mut visitor);
    }

    fn on_new_span(
        &self,
        attrs: &tracing_core::span::Attributes<'_>,
        id: &tracing_core::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Build our json object from the field values like we have been
        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        attrs.record(&mut visitor);

        // And stuff it in our newtype.
        let storage = CustomFieldStorage(fields);

        // Get a reference to the internal span data
        let span = ctx.span(id).unwrap();
        // Get the special place where tracing stores custom data
        let mut extensions = span.extensions_mut();
        // And store our data
        extensions.insert::<CustomFieldStorage>(storage);
    }
}

struct JsonVisitor<'a>(&'a mut BTreeMap<String, serde_json::Value>);

impl<'a> tracing::field::Visit for JsonVisitor<'a> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(value.to_string()),
        );
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(format!("{:?}", value)),
        );
    }
}

struct PrintlnVisitor;

impl tracing::field::Visit for PrintlnVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        println!("  field={} value={:?}", field.name(), value)
    }
}
