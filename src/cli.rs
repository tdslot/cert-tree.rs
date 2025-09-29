use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "cert-tree")]
#[command(about = "X.509 certificate inspection utility")]
#[command(version)]
#[command(after_help = "Github: https://github.com/tdslot/cert-tree.rs")]
pub struct Args {
    /// Certificate file path (PEM or DER)
    #[arg(short, long)]
    pub file: Option<String>,

    /// Certificate URL
    #[arg(short = 'U', long)]
    pub url: Option<String>,

    /// Interactive TUI mode
    #[arg(short = 'i', long, default_value = "false")]
    pub interactive: bool,

    /// Force text output mode (non-interactive)
    #[arg(short = 't', long, default_value = "true")]
    pub text: bool,
}

pub fn parse_args() -> Args {
    let args = Args::parse();

    // If no input arguments provided, show help
    if args.file.is_none() && args.url.is_none() {
        Args::command().print_help().unwrap();
        std::process::exit(0);
    }

    args
}
