use rhai_common::environment::{native::NativeEnvironment, Environment};

use crate::{args::LspCommand, Rhai};

impl<E: Environment> Rhai<E> {
    pub async fn execute_lsp(&self, cmd: LspCommand) -> Result<(), anyhow::Error> {
        let server = rhai_lsp::create_server();
        let world = rhai_lsp::create_world(NativeEnvironment);

        match cmd {
            LspCommand::Tcp { address } => {
                server
                    .listen_tcp(world, &address, async_ctrlc::CtrlC::new().unwrap())
                    .await
            }
            LspCommand::Stdio {} => {
                server
                    .listen_stdio(world, async_ctrlc::CtrlC::new().unwrap())
                    .await
            }
        }
    }
}
