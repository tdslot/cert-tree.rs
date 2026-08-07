//! # cert-tree.rs — read-only X.509 certificate inspection
//!
//! Inspects PEM/DER files and HTTPS endpoints as text, an interactive TUI, or
//! stable JSON schema v1. Relationship construction is not PKIX path validation
//! and does not establish trust or check revocation.

#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::too_many_lines)]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cli;
mod completions;
mod display;
mod error;
mod io;
mod models;
mod parser;
mod report;
mod tree;

use chrono::Utc;
use clap::CommandFactory;
use cli::{parse_args, Args, Commands, CompletionCommands, OutputFormat};
use completions::{generate_completion, install_completion};
use display::{
    display_certificate_tree_text, display_certificate_tree_tui, display_tui, display_verbose,
};
use error::CertError;
use io::{fetch_certificate_chain_from_url, load_certificate_from_file};
use models::{CertificateInfo, CertificateTree};
use parser::parse_certificate_chain;
use report::{policy_failed, write_json, InputKind, InspectionReport};
use std::process::ExitCode;
use tree::build_certificate_tree_at;

fn main() -> ExitCode {
    if std::env::args_os().len() == 1 {
        let mut command = Args::command();
        if command.print_help().is_err() {
            return ExitCode::from(1);
        }
        println!();
        return ExitCode::SUCCESS;
    }

    let args = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            let success = matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            );
            let _ = error.print();
            return ExitCode::from(u8::from(!success));
        }
    };
    if let Err(message) = args.validate() {
        eprintln!("Error: {message}");
        return ExitCode::from(1);
    }
    match run(&args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, CertError> {
    if let Some(command) = &args.command {
        return run_command(command);
    }

    let certificates = load_certificates(args)?;
    let now = Utc::now();
    let tree = build_certificate_tree_at(&certificates, now);
    emit_output(args, &certificates, &tree, now)?;

    if args.check && policy_failed(&tree) {
        eprintln!("Inspection policy found certificate validity or chain construction problems.");
        return Ok(ExitCode::from(2));
    }
    Ok(ExitCode::SUCCESS)
}

fn run_command(command: &Commands) -> Result<ExitCode, CertError> {
    match command {
        Commands::Completion(CompletionCommands::Generate { shell }) => generate_completion(*shell),
        Commands::Completion(CompletionCommands::Install { shell }) => {
            let message = install_completion(*shell).map_err(CertError::InvalidArguments)?;
            println!("{message}");
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn load_certificates(args: &Args) -> Result<Vec<CertificateInfo>, CertError> {
    if let Some(file) = args.file.as_deref() {
        return parse_certificate_chain(&load_certificate_from_file(file)?);
    }
    if let Some(url) = args.url.as_deref() {
        return fetch_certificate_chain_from_url(url);
    }
    Err(CertError::InvalidArguments(
        "one of --file or --url is required".to_string(),
    ))
}

fn emit_output(
    args: &Args,
    certificates: &[CertificateInfo],
    tree: &CertificateTree,
    now: chrono::DateTime<Utc>,
) -> Result<(), CertError> {
    match args.effective_format() {
        OutputFormat::Json => {
            let (kind, source) = if let Some(file) = args.file.as_deref() {
                (InputKind::File, file)
            } else {
                (InputKind::Url, args.url.as_deref().unwrap_or_default())
            };
            let report = InspectionReport::new(kind, source, certificates, tree, now);
            write_json(&mut std::io::stdout().lock(), &report)
        }
        OutputFormat::Text if args.interactive => {
            if tree.roots.len() == 1 && tree.roots[0].children.is_empty() {
                display_tui(&tree.roots[0])?;
            } else {
                display_certificate_tree_tui(tree)?;
            }
            Ok(())
        }
        OutputFormat::Text => {
            if tree.roots.len() == 1 && tree.roots[0].children.is_empty() {
                display_verbose(&tree.roots[0]);
            } else {
                display_certificate_tree_text(tree);
            }
            Ok(())
        }
    }
}
