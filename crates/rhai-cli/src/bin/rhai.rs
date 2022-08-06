use clap::Parser;
use rhai_cli::{
    args::{Colors, RhaiArgs},
    Rhai,
};
use rhai_common::{environment::native::NativeEnvironment, log::setup_stderr_logging};
use std::process::exit;
use tracing::Instrument;

#[tokio::main]
async fn main() {
    let cli = RhaiArgs::parse();
    setup_stderr_logging(
        NativeEnvironment,
        cli.log_spans,
        cli.verbose,
        match cli.colors {
            Colors::Auto => None,
            Colors::Always => Some(true),
            Colors::Never => Some(false),
        },
    );

    match Rhai::new(NativeEnvironment)
        .execute(cli)
        .instrument(tracing::info_span!("rhai"))
        .await
    {
        Ok(_) => {
            exit(0);
        }
        Err(error) => {
            tracing::error!(error = %format!("{error:#}"), "operation failed");
            exit(1);
        }
    }
}
