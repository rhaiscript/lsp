use clap::{App, AppSettings, Arg, ArgMatches};
use rhai_lsp::{create_server, create_world};
use std::{io, process};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    let matches = App::new("rhai")
        .version(env!("CARGO_PKG_VERSION"))
        .author("tamasfe <me@tamasfe.dev>")
        .about("Rhai command-line tool")
        .arg(
            Arg::new("no-colors")
                .long("no-colors")
                .about("Disable output colors")
                .global(true)
                .takes_value(false),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .about("Enable verbose log format")
                .global(true)
                .takes_value(false),
        )
        .subcommand(
            App::new("lsp")
                .about("LSP server commands")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("listen")
                        .about("Start the language server")
                        .setting(AppSettings::SubcommandRequiredElseHelp)
                        .subcommand(App::new("stdio").about("Listen on the standard i/o")),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let format = tracing_subscriber::fmt::format()
        .with_ansi(colors(&matches))
        .pretty();

    let verbose = matches.is_present("verbose");

    let span_events = if verbose {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    let registry = tracing_subscriber::registry();

    let env_filter = match EnvFilter::try_from_default_env() {
        Ok(f) => f,
        Err(_) => EnvFilter::default().add_directive(tracing::Level::INFO.into()),
    };

    if verbose {
        registry
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(io::stderr)
                    .with_span_events(span_events)
                    .event_format(format.pretty()),
            )
            .init();
    } else {
        registry
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(io::stderr)
                    .with_span_events(span_events)
                    .event_format(format.compact()),
            )
            .init();
    }

    match matches.subcommand() {
        Some(("lsp", matches)) => lsp_command(matches).await,
        _ => unreachable!(),
    }
}

fn colors(matches: &ArgMatches) -> bool {
    if matches.is_present("no-colors") {
        return false;
    }

    atty::is(atty::Stream::Stdout)
}

async fn lsp_command(matches: &ArgMatches) {
    match matches.subcommand() {
        Some(("listen", listen_matches)) => match listen_matches.subcommand() {
            Some(("stdio", stdio_matches)) => listen_stdio_command(stdio_matches).await,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

async fn listen_stdio_command(_: &ArgMatches) {
    let world = create_world();
    let srv = create_server();

    srv.listen_stdio(world, async_ctrlc::CtrlC::new().unwrap())
        .await
        .unwrap();

    process::exit(0);
}
