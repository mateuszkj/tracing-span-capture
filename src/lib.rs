use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use tracing::field::Field;
use tracing::{Id, Level, Span};
use tracing_subscriber::field::Visit;
use tracing_subscriber::{layer, Layer};

type Storage = Arc<Mutex<Vec<EventLog>>>;

static GLOBAL_DATA: Lazy<Mutex<HashMap<Id, Storage>>> = Lazy::new(Default::default);

pub struct TracingSpanRecorder;

impl<S> Layer<S> for TracingSpanRecorder
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: layer::Context<'_, S>) {
        if let Some(scope) = ctx.event_scope(event) {
            let data = GLOBAL_DATA.lock().unwrap();

            for span in scope {
                if let Some(logs) = data.get(&span.id()) {
                    let mut fields = FieldsVisitor::default();
                    event.record(&mut fields);

                    let e = EventLog {
                        level: *event.metadata().level(),
                        message: fields.fields.remove("message").unwrap_or(String::new()),
                        fields: fields.fields,
                    };
                    logs.lock().unwrap().push(e);

                    return;
                }
            }
        }
    }
}

#[derive(Default)]
struct FieldsVisitor {
    fields: HashMap<&'static str, String>,
}

impl Visit for FieldsVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields.insert(field.name(), format!("{value:?}"));
    }
}

#[derive(Clone, Debug)]
pub struct EventLog {
    pub level: Level,
    pub message: String,
    pub fields: HashMap<&'static str, String>,
}

pub struct RecordedLogs {
    span_id: Id,
    logs: Storage,
}

impl RecordedLogs {
    pub fn new(span: &Span) -> Self {
        let span_id = span.id().expect("span not enabled, missing id");
        let logs: Arc<Mutex<Vec<EventLog>>> = Default::default();

        GLOBAL_DATA
            .lock()
            .unwrap()
            .insert(span_id.clone(), Arc::clone(&logs));

        RecordedLogs { span_id, logs }
    }

    pub fn into_logs(self) -> Vec<EventLog> {
        self.logs.lock().unwrap().clone()
    }
}

impl Drop for RecordedLogs {
    fn drop(&mut self) {
        GLOBAL_DATA.lock().unwrap().remove(&self.span_id);
    }
}
