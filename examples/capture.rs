use tracing::{error, info, span, Level};
use tracing_span_capture::{RecordedLogs, TracingSpanCaptureLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn fun1(a: i32) {
    error!(a, "try capture that");
    info!("and this");
}
fn main() {
    tracing_subscriber::fmt()
        .finish()
        .with(TracingSpanCaptureLayer)
        .init();

    info!("Started");

    let span = span!(Level::INFO, "");
    let record = RecordedLogs::new(&span);
    {
        let _enter = span.enter();
        fun1(5);
    }

    let logs = record.into_logs();
    logs.into_iter().for_each(|e| info!("Captured: {e:?}"));
}
