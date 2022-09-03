pub mod args;
mod execute;

use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use args::RhaiArgs;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{
        self,
        termcolor::{Ansi, NoColor},
    },
};
use figment::{providers::Serialized, Figment};
use itertools::Itertools;
use rhai_common::{config::Config, environment::Environment, util::Normalize};
use rhai_hir::Hir;
use rhai_rowan::{TextRange, util::is_rhai_def, Parser};
use tokio::io::AsyncWriteExt;
use url::Url;

pub struct Rhai<E: Environment> {
    #[allow(dead_code)]
    env: E,
    config: Config,
    colors: bool,
}

impl<E: Environment> Rhai<E> {
    pub fn new(env: E) -> Self {
        Self {
            env,
            config: Config::default(),
            colors: false,
        }
    }

    pub async fn load_config(&mut self, args: &RhaiArgs) -> anyhow::Result<()> {
        let cwd = self
            .env
            .cwd()
            .ok_or_else(|| anyhow!("invalid working directory"))?;

        let config_path = if let Some(config_path) = &args.config {
            Some(config_path.clone())
        } else if let Some(p) = self.env.discover_rhai_config(&cwd) {
            tracing::debug!(path = ?p, "found configuration file");
            Some(p)
        } else {
            None
        };

        let config_path = match config_path {
            Some(p) => {
                if self.env.is_absolute(&p) {
                    p
                } else {
                    cwd.join(p)
                }
            }
            None => return Ok(()),
        };

        tracing::debug!(path = ?config_path, "using configuration file");

        let bytes = self
            .env
            .read_file(&config_path)
            .await
            .context("failed to read configuration file")?;

        self.config = Figment::new()
            .merge(Serialized::defaults(self.config.clone()))
            .merge(Serialized::defaults(
                toml::from_slice::<Config>(&bytes).context("invalid configuration")?,
            ))
            .extract()
            .context("invalid configuration")?;

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(?cwd))]
    async fn collect_files(
        &self,
        cwd: &Path,
        config: &Config,
        print_summary: bool,
    ) -> Result<Vec<PathBuf>, anyhow::Error> {
        let patterns = match config.source.include.clone() {
            Some(patterns) => patterns,
            None => Vec::from([cwd
                .join("**/*.rhai")
                .normalize()
                .to_string_lossy()
                .into_owned()]),
        };

        let patterns = patterns
            .into_iter()
            .unique()
            .map(|p| glob::Pattern::new(&p).map(|_| p))
            .collect::<Result<Vec<_>, _>>()?;

        let files = patterns
            .into_iter()
            .map(|pat| self.env.glob_files(&pat))
            .collect::<Result<Vec<_>, _>>()
            .into_iter()
            .flatten()
            .flatten()
            .map(Normalize::normalize)
            .collect::<Vec<_>>();

        let total = files.len();

        let files = files
            .into_iter()
            .filter(|path| config.source.is_included(path))
            .collect::<Vec<_>>();

        let excluded = total - files.len();

        if print_summary {
            tracing::info!(total, excluded, "found files");
        } else {
            tracing::debug!(total, excluded, "found files");
        }

        Ok(files)
    }

    pub async fn print_parse_errors(
        &self,
        file: &SimpleFile<&str, &str>,
        errors: &[rhai_rowan::parser::ParseError],
    ) -> Result<(), anyhow::Error> {
        let mut out_diag = Vec::<u8>::new();

        let config = codespan_reporting::term::Config::default();

        for error in errors.iter().unique_by(|e| e.range) {
            let diag = Diagnostic::error()
                .with_message("syntax error")
                .with_labels(Vec::from([
                    Label::primary((), std_range(error.range)).with_message(error.to_string())
                ]));

            if self.colors {
                term::emit(&mut Ansi::new(&mut out_diag), &config, file, &diag)?;
            } else {
                term::emit(&mut NoColor::new(&mut out_diag), &config, file, &diag)?;
            }
        }

        let mut stderr = self.env.stderr();

        stderr.write_all(&out_diag).await?;
        stderr.flush().await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn load_hir(&mut self, cwd: &Path) -> Result<Hir, anyhow::Error> {
        let mut hir = Hir::default();
        self.config.prepare(
            &self.env,
            &self.env.cwd().context("invalid working directory")?,
        )?;
        for file in self.collect_files(cwd, &self.config, false).await? {
            let bytes = match self.env.read_file(&file).await {
                Ok(f) => f,
                Err(error) => {
                    tracing::warn!(path = ?file, %error, "failed to read file");
                    continue;
                }
            };

            let src = match String::from_utf8(bytes).context("invalid source code") {
                Ok(f) => f,
                Err(error) => {
                    tracing::warn!(path = ?file, %error, "invalid source code");
                    continue;
                }
            };

            if is_rhai_def(&src) {
                let url = Url::parse(&format!("file://{}", file.to_string_lossy())).unwrap();
                let def = Parser::new(&src).parse_def().into_syntax();
                hir.add_source(&url, &def);
            }
        }
        Ok(hir)
    }
}

fn std_range(range: TextRange) -> Range<usize> {
    let start: usize = u32::from(range.start()) as _;
    let end: usize = u32::from(range.end()) as _;
    start..end
}
