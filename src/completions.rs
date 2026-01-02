//! Shell completion generation module
//!
//! This module provides functionality to generate shell completion scripts
//! for various shells (bash, zsh, fish, PowerShell).

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::Args;

/// Generate shell completion script and write to stdout
///
/// # Arguments
///
/// * `shell` - The target shell for completion generation
///
/// # Examples
///
/// ```bash
/// cert-tree completion bash > cert-tree.bash
/// cert-tree completion zsh > _cert-tree
/// cert-tree completion fish > cert-tree.fish
/// cert-tree completion powershell > _cert-tree.ps1
/// ```
pub fn generate_completion(shell: Shell) {
    let mut cmd = Args::command();
    let bin_name = cmd.get_name().to_string();
    
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_generation() {
        // Test that completion generation doesn't panic
        let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell];
        
        for shell in shells {
            let mut cmd = Args::command();
            let bin_name = cmd.get_name().to_string();
            let mut buffer = Vec::new();
            generate(shell, &mut cmd, bin_name, &mut buffer);
            
            // Verify some output was generated
            assert!(!buffer.is_empty(), "Completion generation should produce output for {:?}", shell);
        }
    }
}