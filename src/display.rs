use crate::models::{
    CertificateDisplayItem, CertificateInfo, CertificateNode, CertificateTree, ValidityStatus,
};
use chrono::DateTime;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::time::Duration;

/// Maximum scroll limit for TUI details pane
const MAX_SCROLL_LIMIT: u16 = 50;

/// Page size for navigation (items per page)
const PAGE_SIZE: usize = 10;

/// Sleep duration in milliseconds for TUI initialization
const SLEEP_MS: u64 = 50;

/// Starting position for date column in text display
const DATE_COLUMN_START: usize = 78;

pub fn display_verbose(cert: &CertificateInfo) {
    println!("Certificate Information:");
    println!("======================");
    let cn = crate::parser::extract_cn(&cert.subject);
    println!("CN: {cn}");
    println!("Issuer: {}", cert.issuer);
    println!("Serial Number: {}", cert.serial_number);
    println!("Validity:");
    println!("  Not Before: {}", cert.not_before);
    println!("  Not After: {}", cert.not_after);
    println!("Public Key Algorithm: {}", cert.public_key_algorithm);
    println!("Signature Algorithm: {}", cert.signature_algorithm);
    println!("Version: {}", cert.version);
    println!("Is CA: {}", cert.is_ca);

    if let Some(ku) = &cert.key_usage {
        println!("Key Usage: {ku}");
    }

    if !cert.subject_alt_names.is_empty() {
        println!("Subject Alternative Names:");
        for san in &cert.subject_alt_names {
            println!("  {san}");
        }
    }

    println!("Extensions:");
    for ext in &cert.extensions {
        println!(
            "  {} ({}) - {}",
            ext.name.as_deref().unwrap_or(&ext.oid),
            if ext.critical {
                "critical"
            } else {
                "non-critical"
            },
            ext.value
        );
    }
}

pub fn display_tui(cert: &CertificateInfo) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let validity_status = ValidityStatus::from_dates(&cert.not_after);

    // Force initial clear and small delay to ensure proper layout on startup
    terminal.clear()?;
    std::thread::sleep(Duration::from_millis(SLEEP_MS));

    // Scroll state for certificate details pane (unused in single cert view)
    #[allow(unused_variables)]
    let details_scroll: u16 = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(10),   // Certificate info
                    Constraint::Length(3), // Footer
                ])
                .split(size);

            // Title block
            let title = Paragraph::new("🔐 Certificate Inspector")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL).title("cert-tree.rs"));
            f.render_widget(title, chunks[0]);

            // Certificate information
            let cn = crate::parser::extract_cn(&cert.subject);
            let sig_explanation =
                crate::parser::explain_signature_algorithm(&cert.signature_algorithm);
            let mut cert_info = vec![
                Line::from(vec![
                    Span::styled("CN: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cn, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Issuer: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.issuer, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Serial: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.serial_number, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Validity: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.not_before, Style::default().fg(Color::White)),
                    Span::raw(" → "),
                    Span::styled(&cert.not_after, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        validity_status.text(),
                        Style::default().fg(validity_status.color()),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Public Key: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        &cert.public_key_algorithm,
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Signature Algorithm: ", Style::default().fg(Color::Blue)),
                    Span::styled(sig_explanation.as_str(), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Version: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.version.to_string(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Is CA: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        cert.is_ca.to_string(),
                        Style::default().fg(if cert.is_ca {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                    ),
                ]),
            ];

            if let Some(ku) = &cert.key_usage {
                cert_info.push(Line::from(vec![
                    Span::styled("Key Usage: ", Style::default().fg(Color::Blue)),
                    Span::styled(ku, Style::default().fg(Color::Magenta)),
                ]));
            }

            if !cert.subject_alt_names.is_empty() {
                cert_info.push(Line::from(vec![
                    Span::styled("Subject Alt Names: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        cert.subject_alt_names.join(", "),
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
            }

            let cert_paragraph = Paragraph::new(cert_info).wrap(Wrap { trim: true }).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Certificate Details"),
            );
            f.render_widget(cert_paragraph, chunks[1]);

            // Footer with instructions
            let footer = Paragraph::new("Press 'q' to quit")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn display_certificate_tree_text(tree: &CertificateTree) {
    let mut sequence_num = 0;
    for (i, root) in tree.roots.iter().enumerate() {
        let prefix = "━ ";
        display_tree_node_text(
            root,
            prefix,
            0,
            &mut sequence_num,
            i == tree.roots.len() - 1,
        );
    }
}

fn display_tree_node_text(
    node: &CertificateNode,
    prefix: &str,
    depth: usize,
    sequence_num: &mut usize,
    _is_last: bool,
) {
    // Increment sequence number for this certificate
    *sequence_num += 1;

    // Fixed column positions - dates should align regardless of tree depth
    let date_column_start: usize = DATE_COLUMN_START; // Fixed position for date column (adjusted for seconds in time format)

    // Get certificate name (without sequence number) - use only CN
    let cn = crate::parser::extract_cn(&node.cert.subject);
    let available_name_space = date_column_start.saturating_sub(prefix.len()) - 5; // Leave space for brackets and content
    let display_name = if cn.len() > available_name_space {
        let truncate_len = if available_name_space > 3 {
            available_name_space - 3
        } else {
            available_name_space
        };
        format!("{}...", cn.chars().take(truncate_len).collect::<String>())
    } else {
        cn.clone()
    };

    // Date is already in the correct format
    let date_str = node.cert.not_after.clone();

    // Calculate exact padding to align date column
    let name_end_pos = prefix.len() + display_name.len();
    let padding_needed = if name_end_pos < date_column_start {
        date_column_start - name_end_pos
    } else {
        1 // minimum space
    };
    let padding = " ".repeat(padding_needed);

    // Color codes for terminal output
    let (status_text, color_code) = match node.validity_status {
        ValidityStatus::Expired => ("EXPIRED", "\x1b[31m"), // Red
        ValidityStatus::ExpiringSoon => ("EXPIRES SOON", "\x1b[33m"), // Yellow
        ValidityStatus::Valid => ("VALID", "\x1b[32m"),     // Green
    };

    // Use white for certificate names, color only the status/date part
    println!(
        "\x1b[37m[{sequence_num}] {prefix}{display_name}{padding}\x1b[0m{color_code}[{status_text}] [until: {date_str}]\x1b[0m"
    );

    // Display children with cascading tree structure
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;

        // Create cascading indentation for child level (4 spaces per level)
        let child_indent = " ".repeat(5 + (depth * 4)); // 5 spaces base + 4 per depth level
        let child_prefix = format!("{child_indent}└ ");

        display_tree_node_text(child, &child_prefix, depth + 1, sequence_num, is_last_child);
    }
}

pub fn display_certificate_tree_tui(
    tree: &CertificateTree,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Flatten the certificate tree into a list
    let certificates = flatten_certificate_tree(tree);
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(0));

    // Scroll state for certificate details pane
    let mut details_scroll: u16 = 0;

    // State to track if details pane is active for focused navigation
    // When active, arrow keys control details scrolling instead of list navigation
    // Toggle with Tab key for better accessibility and usability
    let mut details_pane_active = false;

    // Force initial clear and small delay to ensure proper layout on startup
    terminal.clear()?;
    std::thread::sleep(Duration::from_millis(SLEEP_MS));

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(5),    // Certificate list
                    Constraint::Min(5),    // Certificate details
                    Constraint::Length(3), // Footer
                ])
                .split(size);

            // Title block with version
            let title_text = format!("🔐 Certificate Chain Inspector{:>width$}", env!("CARGO_PKG_VERSION"), width = size.width as usize - 35);
            let title = Paragraph::new(title_text)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("cert-tree.rs"));
            f.render_widget(title, chunks[0]);

            // Calculate dynamic column widths based on terminal size
            let terminal_width = size.width as usize;
            let min_gap = 2; // Minimum gap between columns
            let min_name_width = 8; // Minimum width for certificate names

            // Adaptive date formatting based on terminal width
            let (date_format, date_width) = if terminal_width < 80 {
                ("%m-%d %H:%M", 11)
            } else if terminal_width < 100 {
                ("%Y-%m-%d %H:%M", 16)
            } else {
                ("%Y-%m-%d %H:%M:%S", 19)
            };

            let padding_after_date = 3;

            let list_area = chunks[1];
            let effective_width = (list_area.width as usize).saturating_sub(2); // Subtract border width (1 left + 1 right)
            let available_name_width = effective_width.saturating_sub(date_width + min_gap + padding_after_date + 4).max(min_name_width);

            // Create list items
            let items: Vec<ListItem> = certificates
                .iter()
                .map(|item| {
                    // Truncate long names if necessary
                    let display_name = if item.display_name.len() > available_name_width {
                        if available_name_width > 3 {
                            format!("{}...", item.display_name.chars().take(available_name_width-3).collect::<String>())
                        } else {
                            item.display_name.chars().take(available_name_width).collect::<String>()
                        }
                    } else {
                        item.display_name.clone()
                    };

                    // Reformat date using adaptive format
                    let formatted_date = if let Ok(dt) = DateTime::parse_from_str(&item.valid_until, "%Y-%m-%d %H:%M:%S") {
                        dt.format(date_format).to_string()
                    } else {
                        item.valid_until.clone()
                    };

                    // Create formatted strings for each column
                    let name_part = format!("{display_name:<available_name_width$}");
                    let safe_date_width = date_width.max(formatted_date.len());
                    let date_part = format!("{formatted_date:>safe_date_width$}");

                    let line = Line::from(vec![
                        Span::styled(name_part, Style::default().fg(Color::White)),
                        Span::styled(date_part, Style::default().fg(item.validity_status.color())),
                        Span::raw("   "), // Add 3 spaces padding after date
                    ]);

                    ListItem::new(line)
                })
                .collect();

            // Create the list widget with visual feedback for active state
            let list_title = if details_pane_active {
                "Certificates (Press Tab to activate)"
            } else {
                "Certificates (Active - Use ↑/↓/PgUp/PgDn to navigate)"
            };

            let list_block = if details_pane_active {
                Block::default()
                    .borders(Borders::ALL)
                    .title(list_title)
            } else {
                Block::default()
                    .borders(Borders::ALL)
                    .title(list_title)
                    .border_style(Style::default().fg(Color::Yellow))
            };

            let list = List::new(items)
                .block(list_block)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, list_area, &mut list_state);

            // Certificate details section
            let selected_index = list_state.selected().unwrap_or(0);
            let selected_cert = &certificates[selected_index];
            let cert = &selected_cert.certificate_info;
            let sig_explanation = crate::parser::explain_signature_algorithm(&cert.signature_algorithm);

            let mut details_lines = vec![
                Line::from(vec![
                    Span::styled("Subject: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.subject, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Issuer: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.issuer, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Serial Number: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.serial_number, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Validity Period: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.not_before, Style::default().fg(Color::White)),
                    Span::raw(" → "),
                    Span::styled(&cert.not_after, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Blue)),
                    Span::styled(selected_cert.validity_status.text(), Style::default().fg(selected_cert.validity_status.color())),
                ]),
                Line::from(vec![
                    Span::styled("Chain Validation: ", Style::default().fg(Color::Blue)),
                    Span::styled(selected_cert.validation_status.text(), Style::default().fg(selected_cert.validation_status.color())),
                ]),
                Line::from(vec![
                    Span::styled("Version: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.version.to_string(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Public Key Algorithm: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.public_key_algorithm, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Signature Algorithm: ", Style::default().fg(Color::Blue)),
                    Span::styled(sig_explanation.as_str(), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Is CA: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.is_ca.to_string(), Style::default().fg(if cert.is_ca { Color::Yellow } else { Color::White })),
                ]),
            ];

            if let Some(ku) = &cert.key_usage {
                details_lines.push(Line::from(vec![
                    Span::styled("Key Usage: ", Style::default().fg(Color::Blue)),
                    Span::styled(ku, Style::default().fg(Color::Magenta)),
                ]));
            }

            if !cert.subject_alt_names.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Subject Alternative Names: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.subject_alt_names.join(", "), Style::default().fg(Color::Cyan)),
                ]));
            }

            if !cert.extensions.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Extensions:", Style::default().fg(Color::Blue)),
                ]));
                for ext in &cert.extensions {
                    let ext_name = ext.name.as_deref().unwrap_or(&ext.oid);
                    details_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(ext_name, Style::default().fg(Color::Cyan)),
                        Span::raw(" ("),
                        Span::styled(if ext.critical { "critical" } else { "non-critical" }, Style::default().fg(if ext.critical { Color::Red } else { Color::Green })),
                        Span::raw(")"),
                    ]));
                }
            }

            // Create details paragraph with visual feedback for active state
            let details_title = if details_pane_active {
                "Certificate Details (Active - Use ↑/↓ to scroll)"
            } else {
                "Certificate Details (Press Tab to activate)"
            };

            let details_block = if details_pane_active {
                Block::default()
                    .borders(Borders::ALL)
                    .title(details_title)
                    .border_style(Style::default().fg(Color::Yellow))
            } else {
                Block::default()
                    .borders(Borders::ALL)
                    .title(details_title)
            };

            let details_paragraph = Paragraph::new(details_lines)
                .wrap(Wrap { trim: true })
                .block(details_block)
                .scroll((details_scroll, 0));
            f.render_widget(details_paragraph, chunks[2]);

            // Footer with instructions - dynamic based on details pane state
            let footer_text = if details_pane_active {
                "Tab: Deactivate Details | ↑/↓: Scroll Details | PgUp/PgDn: Navigate List | 'q' Quit | 't' Text Mode"
            } else {
                "↑/↓/PgUp/PgDn: Navigate List | Tab: Activate Details | 'q' Quit | 't' Text Mode"
            };

            let footer = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        // Enhanced Navigation System:
        // - Tab: Toggle details pane activation/deactivation
        // - When details pane inactive: ↑/↓/PgUp/PgDn navigate certificate list
        // - When details pane active: ↑/↓ scroll certificate details, PgUp/PgDn disabled
        // - 'q'/Esc: Quit application
        // - 't': Switch to text mode
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,

                    // Tab key toggles details pane activation
                    KeyCode::Tab => {
                        details_pane_active = !details_pane_active;
                    }

                    // Navigation keys - behavior depends on details pane state
                    KeyCode::Up => {
                        if details_pane_active {
                            // Scroll details up when details pane is active
                            if details_scroll > 0 {
                                details_scroll = details_scroll.saturating_sub(1);
                            }
                        } else {
                            // Navigate list up when details pane is inactive
                            let i = list_state.selected().unwrap_or(0);
                            if i > 0 {
                                list_state.select(Some(i - 1));
                            }
                        }
                    }
                    KeyCode::Down => {
                        if details_pane_active {
                            // Scroll details down when details pane is active
                            if details_scroll < MAX_SCROLL_LIMIT {
                                details_scroll += 1;
                            }
                        } else {
                            // Navigate list down when details pane is inactive
                            let i = list_state.selected().unwrap_or(0);
                            if i < certificates.len() - 1 {
                                list_state.select(Some(i + 1));
                            }
                        }
                    }

                    // Page Up/Page Down for fast list navigation (only when details pane inactive)
                    KeyCode::PageUp => {
                        if !details_pane_active {
                            let i = list_state.selected().unwrap_or(0);
                            let new_index = i.saturating_sub(PAGE_SIZE);
                            list_state.select(Some(new_index));
                        }
                    }
                    KeyCode::PageDown => {
                        if !details_pane_active {
                            let i = list_state.selected().unwrap_or(0);
                            let new_index = (i + PAGE_SIZE).min(certificates.len() - 1);
                            list_state.select(Some(new_index));
                        }
                    }

                    // Text mode switch
                    KeyCode::Char('t') => {
                        // Switch to text mode
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        display_certificate_tree_text(tree);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn flatten_certificate_tree(tree: &CertificateTree) -> Vec<CertificateDisplayItem> {
    let mut certificates = Vec::new();
    let mut line_number = 1;
    for root in &tree.roots {
        flatten_node(root, &mut certificates, 0, &mut line_number);
    }
    certificates
}

fn flatten_node(
    node: &CertificateNode,
    certificates: &mut Vec<CertificateDisplayItem>,
    depth: usize,
    line_number: &mut usize,
) {
    // Get certificate name (CN only)
    let cn = crate::parser::extract_cn(&node.cert.subject);

    // Create indentation based on depth
    let indentation = "  ".repeat(depth);

    // Format display name with bracketed sequence number, indentation, and certificate name
    let display_name = format!("[{line_number}] {indentation}{cn}");

    // Date is already in the correct format (YYYY-MM-DD HH:MM:SS)
    let valid_until = node.cert.not_after.clone();

    certificates.push(CertificateDisplayItem {
        display_name,
        valid_until,
        validity_status: node.validity_status.clone(),
        validation_status: node.validation_status.clone(),
        certificate_info: node.cert.clone(),
    });

    *line_number += 1;

    // Add children
    for child in &node.children {
        flatten_node(child, certificates, depth + 1, line_number);
    }
}
