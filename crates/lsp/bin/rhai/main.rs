use rhai_lsp::{create_server, create_world, environment::native::NativeEnvironment};
use tracing_subscriber::{
    fmt::format::FmtSpan, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    EnvFilter,
};

#[tokio::main]
pub async fn main() {
    setup_stderr_logging(false, false);

    let server = create_server();
    let world = create_world(NativeEnvironment);

    server
        .listen_stdio(world, async_ctrlc::CtrlC::new().unwrap())
        .await
        .unwrap();
}

pub fn setup_stderr_logging(spans: bool, verbose: bool) {
    let span_events = if spans {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    let registry = tracing_subscriber::registry();

    let env_filter = match std::env::var("RUST_LOG").ok() {
        Some(log) => EnvFilter::new(log),
        None => EnvFilter::default().add_directive(tracing::Level::INFO.into()),
    };

    if verbose {
        registry
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_span_events(span_events)
                    .event_format(tracing_subscriber::fmt::format().pretty().with_ansi(false))
                    .with_writer(std::io::stderr),
            )
            .try_init()
            .ok();
    } else {
        registry
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .event_format(
                        tracing_subscriber::fmt::format()
                            .compact()
                            .with_source_location(false)
                            .with_target(false)
                            .without_time()
                            .with_ansi(false),
                    )
                    .without_time()
                    .with_file(false)
                    .with_line_number(false)
                    .with_span_events(span_events)
                    .with_writer(std::io::stderr),
            )
            .try_init()
            .ok();
    }
}
