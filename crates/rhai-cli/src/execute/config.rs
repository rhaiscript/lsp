use std::path::PathBuf;

use anyhow::Context;
use rhai_common::{config::Config, environment::Environment};
use tokio::io::AsyncWriteExt;

use crate::{args::ConfigCommand, Rhai};

impl<E: Environment> Rhai<E> {
    pub async fn execute_config(&self, cmd: ConfigCommand) -> Result<(), anyhow::Error> {
        let mut stdout = self.env.stdout();

        match cmd {
            ConfigCommand::Schema => {
                stdout
                    .write_all(&serde_json::to_vec(&schemars::schema_for!(Config))?)
                    .await?;
            }
            ConfigCommand::Init { output } => {
                let mut p = PathBuf::from(output);

                if !self.env.is_absolute(&p) {
                    p = self.env.cwd().context("invalid working directory")?.join(p);
                }

                if self.env.read_file(&p).await.is_ok() {
                    tracing::info!("already initialized");
                    return Ok(());
                }

                self.env
                    .write_file(
                        &p,
                        toml::to_string_pretty(&Config::default())
                            .unwrap()
                            .as_bytes(),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
