use crate::{
    args::{RhaiArgs, RootCommand},
    Rhai,
};
use rhai_common::environment::Environment;

mod lsp;

impl<E: Environment> Rhai<E> {
    pub async fn execute(&mut self, args: RhaiArgs) -> Result<(), anyhow::Error> {
        match args.cmd {
            RootCommand::Lsp { cmd } => self.execute_lsp(cmd).await,
        }
    }
}
