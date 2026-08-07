use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cert-tree"))
        .args(arguments)
        .output()
        .expect("cert-tree process should start")
}

#[test]
fn test_help_output_contains_examples_and_exit_codes() {
    let output = run(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("EXAMPLES"));
    assert!(stdout.contains("EXIT CODES"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--check"));
}

#[test]
fn test_text_output_shows_construction_status() {
    let output = run(&["--file", "test/single_cert.pem"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SELF-SIGNED"));
    assert!(stdout.contains("SHA-256 Fingerprint"));
}

#[test]
fn test_json_success_and_stream_separation() {
    let output = run(&["--file", "test/single_cert.pem", "--format", "json"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
}

#[test]
fn test_runtime_error_has_code_one_and_empty_stdout() {
    let output = run(&["--file", "test/does-not-exist.pem"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_check_failure_keeps_json_and_returns_two() {
    let output = run(&[
        "--file",
        "test/leaf_without_issuer.pem",
        "--format",
        "json",
        "--check",
    ]);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["construction"]["diagnostics"][0]["status"],
        "missing_issuer"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("policy"));
}

#[test]
fn test_conflicting_inputs_are_rejected() {
    let output = run(&[
        "--file",
        "test/single_cert.pem",
        "--url",
        "https://example.invalid",
    ]);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_interactive_json_is_rejected_before_tui() {
    let output = run(&[
        "--file",
        "test/single_cert.pem",
        "--interactive",
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}

#[test]
fn test_hidden_text_alias_remains_compatible() {
    let output = run(&["--file", "test/single_cert.pem", "--text"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Certificate Information"));
}

#[test]
fn test_no_arguments_prints_help_but_other_missing_input_is_error() {
    let no_arguments = run(&[]);
    assert_eq!(no_arguments.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&no_arguments.stdout).contains("Usage:"));
    assert_eq!(run(&["--check"]).status.code(), Some(1));
    assert_eq!(run(&["--help"]).status.code(), Some(0));
}

#[test]
fn test_completion_without_inspection_flags_succeeds() {
    let output = run(&["completion", "generate", "bash"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn test_inspection_flags_with_completion_subcommand_are_rejected() {
    let after_subcommand = run(&["completion", "generate", "bash", "--check"]);
    assert_eq!(after_subcommand.status.code(), Some(1));
    assert!(!after_subcommand.stderr.is_empty());

    let before_subcommand = run(&["--format", "text", "completion", "generate", "bash"]);
    assert_eq!(before_subcommand.status.code(), Some(1));
    assert!(before_subcommand.stdout.is_empty());
    assert!(!before_subcommand.stderr.is_empty());
}

#[test]
fn test_text_alias_agrees_with_text_format_and_rejects_json() {
    let accepted = run(&[
        "--file",
        "test/single_cert.pem",
        "--text",
        "--format",
        "text",
    ]);
    assert_eq!(accepted.status.code(), Some(0));

    let rejected = run(&[
        "--file",
        "test/single_cert.pem",
        "--text",
        "--format",
        "json",
    ]);
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stdout.is_empty());
}
