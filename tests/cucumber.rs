use cucumber::{StatsWriter, World};
use tracing::level_filters::LevelFilter;
use tracing_span_capture::TracingSpanRecorder;
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

#[derive(Debug, Default, World)]
struct LogsWorld {}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let summary = LogsWorld::cucumber()
        .configure_and_init_tracing(
            format::DefaultFields::new(),
            format::Format::default(),
            |layer| {
                tracing_subscriber::registry()
                    .with(TracingSpanRecorder)
                    .with(LevelFilter::INFO.and_then(layer))
            },
        )
        .run("tests/features/")
        .await;

    assert!(!summary.execution_has_failed());
    assert_eq!(
        summary.scenarios_stats().skipped,
        0,
        "Should be no skipped scenarios"
    );
}
