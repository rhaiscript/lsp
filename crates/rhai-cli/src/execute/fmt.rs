use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use codespan_reporting::files::SimpleFile;
use rhai_common::{environment::Environment, util::Normalize};
use rhai_fmt::format_syntax;

use crate::{args::FmtCommand, Rhai};

impl<E: Environment> Rhai<E> {
    pub async fn execute_fmt(&mut self, cmd: FmtCommand) -> Result<(), anyhow::Error> {
        let cwd = self
            .env
            .cwd()
            .context("invalid working directory")?
            .normalize();

        let hir = self.load_hir(&cwd).await?;

        if let Some(mut files) = cmd.files {
            if self.env.is_dir(Path::new(&files)) {
                files = PathBuf::from(files)
                    .join("**/*.rhai")
                    .normalize()
                    .to_string_lossy()
                    .into();
            }

            self.config.source.include = Some(vec![files]);
            self.config.source.exclude = None;
        }

        self.config.prepare(
            &self.env,
            &self.env.cwd().context("invalid working directory")?,
        )?;

        let files = self.collect_files(&cwd, &self.config, true).await?;

        let mut result = Ok(());

        let mut format_opts = rhai_fmt::Options::default();
        format_opts.update(self.config.fmt.options.clone());

        for path in files {
            let f = self.env.read_file(&path).await?;
            let source = String::from_utf8_lossy(&f).into_owned();

            let parser = rhai_rowan::Parser::new(&source).with_operators(hir.parser_operators());

            let p = if rhai_rowan::util::is_rhai_def(&source) {
                parser.parse_def()
            } else {
                parser.parse_script()
            };

            if !p.errors.is_empty() {
                self.print_parse_errors(
                    &SimpleFile::new(&*path.to_string_lossy(), source.as_str()),
                    &p.errors,
                )
                .await?;

                if !cmd.force {
                    if cmd.check {
                        result = Err(anyhow!("some files had syntax errors"));
                    } else {
                        result = Err(anyhow!(
                            "some files were not formatted due to syntax errors"
                        ));
                    }

                    continue;
                }
            }

            let formatted = format_syntax(p.into_syntax(), format_opts.clone());

            if source != formatted {
                if cmd.check {
                    tracing::error!(?path, "the file is not properly formatted");
                    result = Err(anyhow!("some files were not properly formatted"));
                } else {
                    self.env.write_file(&path, formatted.as_bytes()).await?;
                }
            }
        }

        result
    }
}
