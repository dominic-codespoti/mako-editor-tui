mod config;
mod mako_config;
use mako_config::{known_keys, allowed_values};

use config::{Config, Param};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

use std::io;
use std::time::Duration;

enum Mode {
    Normal,
    EditValue { idx: usize, input: String },
    AddKey { input: String },
    AddCustomKey { input: String },
    AddValue { key: String, input: String },
    ConfirmDelete { idx: usize },
}

fn main() -> Result<(), io::Error> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load config (or start empty)
    let mut cfg = Config::load().unwrap_or_else(|_| Config { params: vec![] });

    // If the file didn't exist and params empty, seed with a couple helpful keys
    if cfg.params.is_empty() {
        cfg.params.push(Param::new("font", "monospace 10"));
        cfg.params.push(Param::new("background-color", "#1d1f21"));
    }

    // Try to save initial state so file exists and attempt initial reload
    let _ = cfg.save();
    let mut last_reload: Option<(bool, String)> = match cfg.reload() {
        Ok(msg) => Some((true, msg)),
        Err(err) => Some((false, err)),
    };

    let mut list_state = ListState::default();
    if !cfg.params.is_empty() {
        list_state.select(Some(0));
    }

    // separate list state for the known-keys chooser used when adding a key
    let mut key_list_state = ListState::default();
    key_list_state.select(Some(0));

    let mut mode = Mode::Normal;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(4),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(" Mako Config Editor ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" — "),
                Span::styled("↑↓/j/k: navigate ", Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled("e/Enter: edit ", Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled("a: add ", Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled("d: delete ", Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled("q: quit", Style::default().fg(Color::Gray)),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Params list OR known-keys chooser when adding a key
            // Build either the params list (normal) or a filtered known-keys list when adding
            let (list, active_is_keys) = match &mode {
                Mode::AddKey { input } => {
                    // build filtered list from known keys using input as a substring filter (case-insensitive)
                    let filter = input.to_lowercase();
                    let mut filtered: Vec<(String, String)> = Vec::new();
                    for (k, d) in known_keys() {
                        if filter.is_empty() || k.to_lowercase().contains(&filter) || d.to_lowercase().contains(&filter) {
                            filtered.push((k.to_string(), d.to_string()));
                        }
                    }
                    let items: Vec<ListItem> = filtered.iter().map(|(k, desc)| {
                        let line = Line::from(vec![Span::styled(k.clone(), Style::default().add_modifier(Modifier::BOLD)), Span::raw(" - "), Span::raw(desc.clone())]);
                        ListItem::new(line)
                    }).collect();
                    (
                        List::new(items)
                            .block(Block::default().title("Known keys").borders(Borders::ALL))
                            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
                        true,
                    )
                }
                _ => {
                    let items: Vec<ListItem> = cfg
                        .params
                        .iter()
                        .map(|p| {
                            let left = format!("{:20}", p.key);
                            let right = p.value.clone();
                            let line = Line::from(vec![
                                Span::raw(left),
                                Span::raw(" = "),
                                Span::styled(right, Style::default().add_modifier(Modifier::BOLD)),
                            ]);
                            ListItem::new(line)
                        })
                        .collect();

                    (
                        List::new(items)
                            .block(Block::default().title("Parameters").borders(Borders::ALL))
                            .highlight_style(
                                Style::default()
                                    .bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        false,
                    )
                }
            };

            if active_is_keys {
                // Ensure key_list_state selected index is within bounds
                // If out-of-bounds, clamp to 0
                // (List widget will ignore invalid selections but we keep state sane)
                f.render_stateful_widget(list, chunks[1], &mut key_list_state);
            } else {
                f.render_stateful_widget(list, chunks[1], &mut list_state);
            }

            // Footer area depends on mode and reload status
            let footer = match &mode {
                Mode::Normal => {
                    let selected = list_state.selected().map(|i| format!("Selected: {} = {}", cfg.params[i].key, cfg.params[i].value)).unwrap_or_else(|| "No selection".to_string());

                    // build status spans
                    let mut spans = vec![
                        Span::raw(selected),
                        Span::raw("    "),
                        Span::styled("Press 'a' to add, 'e' to edit, 'd' to delete.", Style::default().fg(Color::Gray)),
                    ];

                    if let Some((ok, msg)) = &last_reload {
                        if *ok {
                            spans.push(Span::raw("    "));
                            spans.push(Span::styled(format!("Reload OK: {}", msg), Style::default().fg(Color::Green)));
                        } else {
                            spans.push(Span::raw("    "));
                            spans.push(Span::styled(format!("Reload failed: {}", msg), Style::default().fg(Color::Red)));
                        }
                    }

                    Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL))
                }
                Mode::EditValue { idx, input } => {
                    let key = if *idx < cfg.params.len() { cfg.params[*idx].key.clone() } else { "".to_string() };
                    let mut spans = vec![
                        Span::raw("Editing value (Enter=save, Esc=cancel): "),
                        Span::styled(input.clone(), Style::default().add_modifier(Modifier::BOLD)),
                    ];
                    if let Some(vals) = allowed_values(&key) {
                        spans.push(Span::raw("    "));
                        spans.push(Span::styled(format!("Allowed: {}", vals.join(" | ")), Style::default().fg(Color::Gray)));
                    }
                    Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL))
                }
                Mode::AddKey { input } => {
                    Paragraph::new(Line::from(vec![
                        Span::raw("New key (Enter=next, Esc=cancel). Use ↑/↓ to pick, or type to filter: "),
                        Span::styled(input.clone(), Style::default().add_modifier(Modifier::BOLD)),
                    ])).block(Block::default().borders(Borders::ALL))
                }
                Mode::AddCustomKey { input } => {
                    Paragraph::new(Line::from(vec![
                        Span::raw("Custom key name (Enter=next, Esc=cancel): "),
                        Span::styled(input.clone(), Style::default().add_modifier(Modifier::BOLD)),
                    ])).block(Block::default().borders(Borders::ALL))
                }
                Mode::AddValue { key, input } => {
                    let mut spans = vec![
                        Span::raw(format!("Value for '{}' (Enter=add, Esc=cancel): ", key)),
                        Span::styled(input.clone(), Style::default().add_modifier(Modifier::BOLD)),
                    ];
                    if let Some(vals) = allowed_values(&key) {
                        spans.push(Span::raw("    "));
                        spans.push(Span::styled(format!("Allowed: {}", vals.join(" | ")), Style::default().fg(Color::Gray)));
                    }
                    Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL))
                }
                Mode::ConfirmDelete { idx } => {
                    let key = &cfg.params[*idx].key;
                    Paragraph::new(Line::from(vec![
                        Span::styled("Confirm delete? ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw(format!("Delete '{}' (y/n): ", key)),
                    ])).block(Block::default().borders(Borders::ALL))
                }
            };

            f.render_widget(footer, chunks[2]);
        })?;

        // Input handling
        if event::poll(Duration::from_millis(120))? {
            if let CEvent::Key(key) = event::read()? {
                match &mut mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !cfg.params.is_empty() {
                                let i = list_state.selected().unwrap_or(0);
                                let next = (i + 1) % cfg.params.len();
                                list_state.select(Some(next));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if !cfg.params.is_empty() {
                                let i = list_state.selected().unwrap_or(0);
                                let prev = if i == 0 { cfg.params.len() - 1 } else { i - 1 };
                                list_state.select(Some(prev));
                            }
                        }
                        KeyCode::Enter | KeyCode::Char('e') => {
                            if let Some(i) = list_state.selected() {
                                let current = cfg.params[i].value.clone();
                                mode = Mode::EditValue { idx: i, input: current };
                            }
                        }
                        KeyCode::Char('a') => {
                            mode = Mode::AddKey { input: String::new() };
                        }
                        KeyCode::Char('d') => {
                            if let Some(i) = list_state.selected() {
                                mode = Mode::ConfirmDelete { idx: i };
                            }
                        }
                        _ => {}
                    },
                    Mode::EditValue { idx, input } => match key.code {
                        KeyCode::Esc => {
                            mode = Mode::Normal;
                        }
                        KeyCode::Enter => {
                            // commit
                            if *idx < cfg.params.len() {
                                cfg.params[*idx].value = input.clone();
                                if let Ok(_) = cfg.save() {
                                    cfg.notify(&cfg.params[*idx].key, &cfg.params[*idx].value);
                                    // attempt reload and capture result
                                    match cfg.reload() {
                                        Ok(msg) => last_reload = Some((true, msg)),
                                        Err(err) => last_reload = Some((false, err)),
                                    }
                                }
                            }
                            mode = Mode::Normal;
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        _ => {}
                    },
                    Mode::AddKey { input } => match key.code {
                        KeyCode::Esc => {
                            mode = Mode::Normal;
                        }
                        KeyCode::Enter => {
                            // Build filtered key list the same way as the renderer
                            let filter = input.to_lowercase();
                            let filtered: Vec<String> = known_keys()
                                .into_iter()
                                .filter_map(|(k, d)| {
                                    let kl = k.to_lowercase();
                                    let dl = d.to_lowercase();
                                    if filter.is_empty() || kl.contains(&filter) || dl.contains(&filter) {
                                        Some(k.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            // Only accept a selection from the filtered list. If '<custom>' selected, open custom-key prompt.
                            if let Some(i) = key_list_state.selected() {
                                if let Some(k) = filtered.get(i) {
                                    if k == "<custom>" {
                                        mode = Mode::AddCustomKey { input: String::new() };
                                        continue;
                                    } else {
                                        mode = Mode::AddValue { key: k.clone(), input: String::new() };
                                        continue;
                                    }
                                }
                            }

                            // If nothing highlighted (shouldn't happen), return to normal
                            mode = Mode::Normal;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            // move selection over the filtered list shown in the UI
                            let filter = input.to_lowercase();
                            let filtered: Vec<String> = known_keys()
                                .into_iter()
                                .filter_map(|(k, d)| {
                                    let kl = k.to_lowercase();
                                    let dl = d.to_lowercase();
                                    if filter.is_empty() || kl.contains(&filter) || dl.contains(&filter) {
                                        Some(k.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !filtered.is_empty() {
                                let i = key_list_state.selected().unwrap_or(0);
                                let next = (i + 1) % filtered.len();
                                key_list_state.select(Some(next));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let filter = input.to_lowercase();
                            let filtered: Vec<String> = known_keys()
                                .into_iter()
                                .filter_map(|(k, d)| {
                                    let kl = k.to_lowercase();
                                    let dl = d.to_lowercase();
                                    if filter.is_empty() || kl.contains(&filter) || dl.contains(&filter) {
                                        Some(k.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !filtered.is_empty() {
                                let i = key_list_state.selected().unwrap_or(0);
                                let prev = if i == 0 { filtered.len() - 1 } else { i - 1 };
                                key_list_state.select(Some(prev));
                            }
                        }
                        KeyCode::Backspace => { input.pop(); }
                        KeyCode::Char(c) => { input.push(c); }
                        _ => {}
                    },
                    Mode::AddCustomKey { input } => match key.code {
                        KeyCode::Esc => { mode = Mode::Normal; }
                        KeyCode::Enter => {
                            let keyname = input.trim().to_string();
                            if !keyname.is_empty() {
                                mode = Mode::AddValue { key: keyname, input: String::new() };
                            } else {
                                mode = Mode::Normal;
                            }
                        }
                        KeyCode::Backspace => { input.pop(); }
                        KeyCode::Char(c) => { input.push(c); }
                        _ => {}
                    },
                    Mode::AddValue { key: key_str, input } => match key.code {
                        KeyCode::Esc => {
                            mode = Mode::Normal;
                        }
                        KeyCode::Enter => {
                            let val = input.clone();
                            if !key_str.trim().is_empty() {
                                cfg.add_param(key_str.clone(), val.clone());
                                if let Ok(_) = cfg.save() {
                                    cfg.notify(&key_str, &val);
                                    match cfg.reload() {
                                        Ok(msg) => last_reload = Some((true, msg)),
                                        Err(err) => last_reload = Some((false, err)),
                                    }
                                }
                                // select the newly added item
                                let last = cfg.params.len() - 1;
                                list_state.select(Some(last));
                            }
                            mode = Mode::Normal;
                        }
                        KeyCode::Backspace => { input.pop(); }
                        KeyCode::Char(c) => { input.push(c); }
                        _ => {}
                    },
                    Mode::ConfirmDelete { idx } => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            if *idx < cfg.params.len() {
                                let removed = cfg.params[*idx].key.clone();
                                cfg.remove_param(*idx);
                                if let Ok(_) = cfg.save() {
                                    cfg.notify(&removed, "<deleted>");
                                    match cfg.reload() {
                                        Ok(msg) => last_reload = Some((true, msg)),
                                        Err(err) => last_reload = Some((false, err)),
                                    }
                                }
                                // adjust selection
                                if cfg.params.is_empty() {
                                    list_state.select(None);
                                } else {
                                    let new_idx = if *idx == 0 { 0 } else { *idx - 1 };
                                    list_state.select(Some(new_idx));
                                }
                            }
                            mode = Mode::Normal;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            mode = Mode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
