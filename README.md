# Capture and record logs for tracing span id

This crate allow testing if function emitted logs.

## Example

Example how to use it with [cucumber](https://cucumber-rs.github.io/cucumber/main/), you can find [here](./tests/cucumber.rs).

```rust
use tracing::{error, span, Level};
use tracing_span_capture::{RecordedLogs, TracingSpanCaptureLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn tested_code() {
    error!("try capture this");
}

fn main() {
    tracing_subscriber::fmt()
        .finish()
        .with(TracingSpanCaptureLayer)
        .init();

    let span = span!(Level::INFO, "");
    let record = RecordedLogs::new(&span);
    {
        let _enter = span.enter();
        tested_code();
    }
    
    let logs = record.into_logs();
    let last_log = logs.into_iter().rev().next().unwrap();
    assert_eq!(last_log.message, "try capture this");
}
```

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)), or
* MIT license ([LICENSE-MIT](LICENSE-MIT))
