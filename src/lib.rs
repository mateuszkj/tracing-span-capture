//! This crate allows testing code that should emit logs.
//! It do that by capturing and recording logs for a given tracing span id.
//!
//! # Examples
//!
//! ```no_run
//! use tracing::{error, span, Level};
//! use tracing_span_capture::{RecordedLogs, TracingSpanCaptureLayer};
//! use tracing_subscriber::layer::SubscriberExt;
//! use tracing_subscriber::util::SubscriberInitExt;
//!
//! tracing_subscriber::fmt()
//!     .finish()
//!     .with(TracingSpanCaptureLayer)
//!     .init();
//!
//! let span = span!(Level::INFO, "");
//! let record = RecordedLogs::new(&span);
//! {
//!     let _enter = span.enter();
//!     error!("try capture this");
//! }
//!
//! let logs = record.into_logs();
//! let last_log = logs.into_iter().rev().next().unwrap();
//! assert_eq!(last_log.message, "try capture this");
//! ```
//!
#![warn(missing_docs)]
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::{fmt, mem};
use tracing::field::Field;
use tracing::{Id, Level, Span};
use tracing_subscriber::field::Visit;
use tracing_subscriber::{layer, Layer};

type Storage = Arc<Mutex<Vec<EventLog>>>;

static GLOBAL_DATA: Lazy<Mutex<HashMap<Id, Storage>>> = Lazy::new(Default::default);

/// Tracing Subscriber layer which has to be registered globally
///
/// # Examples
///
/// ```no_run
/// use tracing_subscriber::layer::SubscriberExt;
/// use tracing_span_capture::TracingSpanCaptureLayer;
/// use tracing_subscriber::util::SubscriberInitExt;
///
/// tracing_subscriber::fmt()
///     .finish()
///     .with(TracingSpanCaptureLayer)
///     .init();
/// ```
pub struct TracingSpanCaptureLayer;

impl<S> Layer<S> for TracingSpanCaptureLayer
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

/// Captured log event
#[derive(Clone, Debug)]
pub struct EventLog {
    /// Log Level
    pub level: Level,

    /// Emitted message from log event
    pub message: String,

    /// Emitted fields from log event
    pub fields: HashMap<&'static str, String>,
}

/// Handler for captured logs storage
///
/// ```no_run
/// use tracing::{span, Level};
/// use tracing_span_capture::RecordedLogs;
///
/// let span = span!(Level::INFO, "");
/// let logs = RecordedLogs::new(&span);
/// {
///     let _enter = span.enter();
/// }
/// let _logs_list = logs.into_logs();
/// ```
pub struct RecordedLogs {
    span_id: Id,
    logs: Storage,
}

impl RecordedLogs {
    /// Initialize logs storage for given span id
    ///
    /// # Panics
    ///
    /// Panics If span has been either closed or was never enabled
    pub fn new(span: &Span) -> Self {
        let span_id = span.id().expect("span not enabled, missing id");
        let logs: Arc<Mutex<Vec<EventLog>>> = Default::default();

        GLOBAL_DATA
            .lock()
            .unwrap()
            .insert(span_id.clone(), Arc::clone(&logs));

        RecordedLogs { span_id, logs }
    }

    /// Take logs from storage
    pub fn into_logs(self) -> Vec<EventLog> {
        let mut storage = self.logs.lock().unwrap();
        let logs = mem::take(storage.deref_mut());
        logs
    }
}

impl Drop for RecordedLogs {
    fn drop(&mut self) {
        GLOBAL_DATA.lock().unwrap().remove(&self.span_id);
    }
}
