use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "cert-tree")]
#[command(about = "X.509 certificate inspection utility")]
#[command(version)]
#[command(after_help = "Github: https://github.com/tdslot/cert-tree.rs")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Certificate file path (PEM or DER)
    #[arg(short, long, global = true)]
    pub file: Option<String>,

    /// Certificate URL
    #[arg(short = 'U', long, global = true)]
    pub url: Option<String>,

    /// Interactive TUI mode
    #[arg(short = 'i', long, default_value = "false", global = true)]
    pub interactive: bool,

    /// Force text output mode (non-interactive)
    #[arg(short = 't', long, default_value = "true", global = true)]
    pub text: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage shell completions
    #[command(subcommand)]
    Completion(CompletionCommands),
}

#[derive(Subcommand)]
pub enum CompletionCommands {
    /// Generate shell completion script to stdout
    ///
    /// Examples:
    ///   cert-tree completion generate bash > cert-tree.bash
    ///   cert-tree completion generate zsh > _cert-tree
    ///   cert-tree completion generate fish > cert-tree.fish
    Generate {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Install shell completion (auto-detects shell)
    ///
    /// Automatically detects your current shell and installs completion
    /// to the appropriate location. You can also specify a shell explicitly.
    ///
    /// Examples:
    ///   cert-tree completion install              # Auto-detect and install
    ///   cert-tree completion install --shell bash # Install for bash
    Install {
        /// Shell type (optional, will auto-detect if not provided)
        #[arg(short, long, value_enum)]
        shell: Option<Shell>,
    },
}

pub fn parse_args() -> Args {
    let args = Args::parse();

    // If subcommand is provided, it's handled in main
    if args.command.is_some() {
        return args;
    }

    // If no input arguments provided, show help
    if args.file.is_none() && args.url.is_none() {
        Args::command().print_help().unwrap();
        std::process::exit(0);
    }

    args
}
