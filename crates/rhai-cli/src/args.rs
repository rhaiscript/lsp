use clap::{crate_version, ArgEnum, Parser, Subcommand};

#[derive(Clone, Parser)]
#[clap(name = "rhai")]
#[clap(bin_name = "rhai")]
#[clap(version = crate_version!())]
pub struct RhaiArgs {
    #[clap(long, arg_enum, global = true, default_value = "auto")]
    pub colors: Colors,
    /// Enable a verbose logging format.
    #[clap(long, global = true)]
    pub verbose: bool,
    /// Enable logging spans.
    #[clap(long, global = true)]
    pub log_spans: bool,
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
    Stdio {},
}

#[derive(Clone, Copy, ArgEnum)]
pub enum Colors {
    /// Determine whether to colorize output automatically.
    Auto,
    /// Always colorize output.
    Always,
    /// Never colorize output.
    Never,
}
