//! Command-line interface definition and validation.

use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Inspection output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Parser)]
#[command(name = "cert-tree")]
#[command(about = "Read-only X.509 certificate inspection utility")]
#[command(version)]
#[command(long_about = "Read-only X.509 certificate inspection utility.\n\n\
                  Parses PEM/DER certificate files and fetches certificate chains from HTTPS \
                  endpoints, then displays them as a human-readable tree, an interactive TUI, \
                  or a stable JSON report suitable for scripting and CI.")]
#[command(after_help = "EXAMPLES:\n    \
                  cert-tree --file cert.pem\n    \
                  cert-tree --file chain.pem --interactive\n    \
                  cert-tree --url https://example.com\n    \
                  cert-tree --file cert.pem --format json\n    \
                  cert-tree --file chain.pem --check --format json\n    \
                  cert-tree completion install\n\n\
                  EXIT CODES:\n    \
                  0  inspection succeeded (and --check found no problem)\n    \
                  1  CLI, input, parse, network, or output error\n    \
                  2  output succeeded, but --check found a validity or construction problem\n\n\
                  NOTE:\n    \
                  Chain output shows issuer/subject *construction* diagnostics (LINKED,\n    \
                  MISSING ISSUER, AMBIGUOUS ISSUER, SELF-SIGNED). It is not PKIX trust\n    \
                  path validation and performs no revocation (OCSP/CRL) checks.\n\n\
                  GitHub: https://github.com/tdslot/cert-tree.rs")]
#[command(disable_help_subcommand = true)]
#[command(group(ArgGroup::new("input").args(["file", "url"]).multiple(false)))]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Certificate file path (PEM or DER)
    #[arg(
        short,
        long,
        long_help = "Path to a local certificate file.\n\n\
                     Supports PEM bundles (one or more certificates) and DER files.\n\
                     Multiple PEM certificates are shown as a certificate chain tree.\n\
                     Mutually exclusive with --url."
    )]
    pub file: Option<String>,

    /// Certificate file URL or HTTPS endpoint
    #[arg(
        short = 'U',
        long,
        long_help = "HTTPS URL to inspect.\n\n\
                     If the URL serves a PEM or DER certificate, it is downloaded and parsed.\n\
                     Otherwise a TLS handshake is performed and the presented certificate\n\
                     chain is inspected. Only the https:// scheme is supported; the\n\
                     connection always uses port 443. Mutually exclusive with --file."
    )]
    pub url: Option<String>,

    /// Interactive TUI mode
    #[arg(
        short,
        long,
        long_help = "Open an interactive terminal UI.\n\n\
                     Navigate the certificate list with arrow keys, switch panes with Tab,\n\
                     quit with 'q'. Cannot be combined with --format json or --check."
    )]
    pub interactive: bool,

    /// Output format
    #[arg(
        long,
        value_enum,
        long_help = "Output format [default: text].\n\n\
                     text: human-readable certificate tree or verbose details.\n\
                     json: stable machine-readable report (schema_version: 1) written to\n\
                     stdout only, suitable for jq, scripting, and CI. Diagnostics and\n\
                     errors always go to stderr."
    )]
    pub format: Option<OutputFormat>,

    /// Return exit code 2 when validity or construction checks find problems
    #[arg(
        long,
        long_help = "Enable policy checking after the report is printed.\n\n\
                     The report is always emitted first; then exit code 2 is returned if any\n\
                     certificate is expired, not yet valid, expiring within 30 days, has an\n\
                     unknown validity, or its issuer is missing or ambiguous. LINKED and\n\
                     SELF-SIGNED pass. Intended for monitoring and CI pipelines."
    )]
    pub check: bool,

    /// Deprecated compatibility alias for text output
    #[arg(short = 't', long, hide = true)]
    pub text: bool,
}

impl Args {
    /// Effective output format, defaulting to human-readable text.
    pub fn effective_format(&self) -> OutputFormat {
        self.format.unwrap_or(OutputFormat::Text)
    }

    /// Validate combinations whose validity depends on values.
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_some() {
            if self.file.is_some()
                || self.url.is_some()
                || self.interactive
                || self.check
                || self.text
                || self.format.is_some()
            {
                return Err("inspection options cannot be combined with a subcommand".to_string());
            }
            return Ok(());
        }
        if self.file.is_none() && self.url.is_none() {
            return Err("one of --file or --url is required".to_string());
        }
        if self.text && self.effective_format() == OutputFormat::Json {
            return Err("--text cannot be combined with --format json".to_string());
        }
        if self.interactive && self.effective_format() == OutputFormat::Json {
            return Err("--interactive cannot be combined with --format json".to_string());
        }
        if self.interactive && self.check {
            return Err("--interactive cannot be combined with --check".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Manage shell completions
    #[command(subcommand)]
    Completion(CompletionCommands),
}

#[derive(Debug, Subcommand)]
pub enum CompletionCommands {
    /// Generate shell completion script to stdout
    #[command(
        long_about = "Generate a shell completion script and print it to stdout.\n\n\
                      Redirect it to the appropriate completion directory for your shell,\n\
                      or use `cert-tree completion install` for automatic installation."
    )]
    Generate {
        /// Shell type
        #[arg(
            value_enum,
            long_help = "Shell to generate completions for: bash, zsh, fish, or powershell."
        )]
        shell: Shell,
    },
    /// Install shell completion (auto-detects shell)
    #[command(long_about = "Install the shell completion script for your shell.\n\n\
                      The shell is auto-detected from the environment when --shell is omitted.\n\
                      Restart the shell or source the completion file after installation.")]
    Install {
        /// Shell type (auto-detected if omitted)
        #[arg(
            short,
            long,
            value_enum,
            long_help = "Shell to install completions for: bash, zsh, fish, or powershell. Auto-detected when omitted."
        )]
        shell: Option<Shell>,
    },
}

pub fn parse_args() -> Result<Args, clap::Error> {
    Args::try_parse()
}
