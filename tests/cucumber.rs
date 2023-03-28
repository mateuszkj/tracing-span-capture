use cucumber::{then, when, StatsWriter, World};
use tracing::level_filters::LevelFilter;
use tracing::{error, span, Level};
use tracing_span_capture::{RecordedLogs, TracingSpanCaptureLayer};
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

#[derive(Debug, Default, World)]
struct LogsWorld {
    last_error_message: Option<String>,
}

fn tested_function() {
    error!("abc");
    error!("emit test1");
}

#[when("Function is called")]
fn call_function(world: &mut LogsWorld) {
    let span = span!(Level::INFO, "");
    let record = RecordedLogs::new(&span);
    {
        let _enter = span.enter();
        tested_function();
    }

    let logs = record.into_logs();
    world.last_error_message = logs
        .into_iter()
        .rev()
        .find(|e| e.level == Level::ERROR)
        .map(|e| e.message);
}

#[then(expr = "Error log {string} is emitted")]
fn then_error(world: &mut LogsWorld, text: String) {
    let last_error = world
        .last_error_message
        .as_ref()
        .expect("no error was emitted");
    assert!(
        last_error.contains(&text),
        "Expected log containing {text} got: {last_error}"
    );
}

#[then(expr = "Error log {string} is not emitted")]
fn then_not_error(world: &mut LogsWorld, text: String) {
    if let Some(last_error) = &world.last_error_message {
        assert!(
            !last_error.contains(&text),
            "Expected log not containing {text} got: {last_error}"
        );
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let summary = LogsWorld::cucumber()
        .configure_and_init_tracing(
            format::DefaultFields::new(),
            format::Format::default(),
            |layer| {
                tracing_subscriber::registry()
                    .with(TracingSpanCaptureLayer)
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
