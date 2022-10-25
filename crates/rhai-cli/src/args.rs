use std::path::PathBuf;

use clap::{crate_version, Parser, Subcommand, ValueEnum};

#[derive(Clone, Parser)]
#[clap(name = "rhai")]
#[clap(bin_name = "rhai")]
#[clap(version = crate_version!())]
pub struct RhaiArgs {
    #[clap(long, global = true, default_value = "auto")]
    pub colors: Colors,
    /// Enable a verbose logging format.
    #[clap(long, global = true)]
    pub verbose: bool,
    /// Enable logging spans.
    #[clap(long, global = true)]
    pub log_spans: bool,
    /// Path to `Rhai.toml` configuration file.
    #[clap(long, global = true)]
    pub config: Option<PathBuf>,
    #[clap(subcommand)]
    pub cmd: RootCommand,
}

#[derive(Clone, Subcommand)]
pub enum RootCommand {
    /// Language server operations.
    Lsp {
        #[clap(subcommand)]
        cmd: LspCommand,
    },
    /// Configuration file operations
    #[clap(visible_aliases = &["cfg"])]
    Config {
        #[clap(subcommand)]
        cmd: ConfigCommand,
    },
    /// Format Rhai source code.
    #[clap(visible_aliases = &["format"])]
    Fmt(FmtCommand),
}

#[derive(Clone, Subcommand)]
pub enum LspCommand {
    /// Run the language server and listen on a TCP address.
    Tcp {
        /// The address to listen on.
        #[clap(long, default_value = "0.0.0.0:9181")]
        address: String,
    },
    /// Run the language server over the standard input and output.
    Stdio,
}

#[derive(Clone, Subcommand)]
pub enum ConfigCommand {
    /// Print the configuration JSON schema.
    Schema,
    /// Create a new configuration file with default values.
    Init {
        /// Output file path.
        #[clap(short = 'o', long, default_value = "Rhai.toml")]
        output: String,
    },
}

#[derive(Clone, Parser)]
pub struct FmtCommand {
    /// Proceed with formatting even if the files contain
    /// syntax errors.
    #[clap(short, long)]
    pub force: bool,

    /// Dry-run and report any files that are not correctly formatted.
    #[clap(long)]
    pub check: bool,

    /// Optional pattern to search for files.
    ///
    /// If not provided, it will be determined by
    /// the configuration.
    pub files: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Colors {
    /// Determine whether to colorize output automatically.
    Auto,
    /// Always colorize output.
    Always,
    /// Never colorize output.
    Never,
}
