//! Shell completion generation module
//!
//! This module provides functionality to generate and install shell completion scripts
//! for various shells (bash, zsh, fish, PowerShell).

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

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

/// Detect the current shell from environment variables
///
/// Returns the detected shell or None if unable to detect
pub fn detect_shell() -> Option<Shell> {
    // Check SHELL environment variable (Unix-like systems)
    if let Ok(shell_path) = env::var("SHELL") {
        if shell_path.contains("bash") {
            return Some(Shell::Bash);
        } else if shell_path.contains("zsh") {
            return Some(Shell::Zsh);
        } else if shell_path.contains("fish") {
            return Some(Shell::Fish);
        }
    }
    
    // Check for PowerShell on Windows
    if cfg!(windows) {
        return Some(Shell::PowerShell);
    }
    
    None
}

/// Get the default installation path for shell completion
///
/// # Arguments
///
/// * `shell` - The target shell
///
/// # Returns
///
/// The default installation path or None if unable to determine
pub fn get_completion_path(shell: Shell) -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    
    match shell {
        Shell::Bash => {
            // Try user-local first, then system-wide
            if cfg!(target_os = "macos") {
                Some(PathBuf::from("/usr/local/etc/bash_completion.d/cert-tree"))
            } else {
                Some(PathBuf::from(format!("{}/.local/share/bash-completion/completions/cert-tree", home)))
            }
        }
        Shell::Zsh => {
            Some(PathBuf::from(format!("{}/.zsh/completion/_cert-tree", home)))
        }
        Shell::Fish => {
            Some(PathBuf::from(format!("{}/.config/fish/completions/cert-tree.fish", home)))
        }
        Shell::PowerShell => {
            // PowerShell profile location
            None // PowerShell requires adding to profile, not a completion file
        }
        _ => None,
    }
}

/// Install completion script to the appropriate location
///
/// # Arguments
///
/// * `shell` - The target shell (if None, will auto-detect)
///
/// # Returns
///
/// Result indicating success or error message
pub fn install_completion(shell: Option<Shell>) -> Result<String, String> {
    let detected_shell = match shell {
        Some(s) => s,
        None => detect_shell().ok_or("Unable to detect shell. Please specify shell explicitly.")?,
    };
    
    // Special handling for PowerShell
    if matches!(detected_shell, Shell::PowerShell) {
        return Err("PowerShell completion requires manual setup. Run: cert-tree completion powershell >> $PROFILE".to_string());
    }
    
    let install_path = get_completion_path(detected_shell)
        .ok_or("Unable to determine installation path for this shell")?;
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = install_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    // Generate completion script
    let mut cmd = Args::command();
    let bin_name = cmd.get_name().to_string();
    let mut buffer = Vec::new();
    generate(detected_shell, &mut cmd, bin_name, &mut buffer);
    
    // Write to file
    fs::write(&install_path, buffer)
        .map_err(|e| format!("Failed to write completion file: {}", e))?;
    
    let shell_name = format!("{:?}", detected_shell).to_lowercase();
    let path_str = install_path.display();
    
    // Provide post-install instructions
    let instructions = match detected_shell {
        Shell::Bash => {
            if cfg!(target_os = "macos") {
                "Reload your shell: source ~/.bash_profile"
            } else {
                "Reload your shell: source ~/.bashrc"
            }
        }
        Shell::Zsh => {
            "Add to ~/.zshrc if not present:\nfpath=(~/.zsh/completion $fpath)\nautoload -Uz compinit && compinit\n\nThen reload: source ~/.zshrc"
        }
        Shell::Fish => {
            "Completions will be available in new fish shells automatically"
        }
        _ => "Restart your shell to activate completions",
    };
    
    Ok(format!(
        "✓ Shell completion installed successfully!\n\n\
        Shell: {}\n\
        Location: {}\n\n\
        {}\n",
        shell_name, path_str, instructions
    ))
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