use crate::{
    args::{Colors, RhaiArgs, RootCommand},
    Rhai,
};
use rhai_common::environment::Environment;

mod config;
mod fmt;
mod lsp;

impl<E: Environment> Rhai<E> {
    pub async fn execute(&mut self, args: RhaiArgs) -> Result<(), anyhow::Error> {
        if let RootCommand::Fmt(_) = &args.cmd {
            self.load_config(&args).await?
        }

        self.colors = match args.colors {
            Colors::Auto => self.env.atty_stderr(),
            Colors::Always => true,
            Colors::Never => false,
        };

        match args.cmd {
            RootCommand::Lsp { cmd } => self.execute_lsp(cmd).await,
            RootCommand::Config { cmd } => self.execute_config(cmd).await,
            RootCommand::Fmt(cmd) => self.execute_fmt(cmd).await,
        }
    }
}
