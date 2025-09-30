Mako TUI
========

A small terminal user interface for editing mako notification daemon configuration files.

This project provides a curses-style TUI (built with crossterm + ratatui) to edit mako-style key/value configuration parameters. It includes a typed model of common mako configuration keys in `src/mako_config.rs` and a simple list editor in `src/main.rs`.

Features
--------

- Navigate parameters in a list.
- Edit existing values with inline input and suggested allowed values for known keys.
- Add new keys from a curated list of known mako keys or create a custom key.
- Delete parameters with confirmation.
- Basic file save and reload feedback.

Quickstart
----------

Prerequisites
- Rust (stable) toolchain installed (rustup)

Build and run

Run the app in the repo root:

# Mako TUI

A small terminal UI to edit mako (notification daemon) configuration key/value pairs.

This crate bundles a simple TUI editor built with `crossterm` + `ratatui`. It provides
a curated list of common mako options (see `src/mako_config.rs`), inline editing with
allowed-value hints, and a small `Config` loader/saver used by the UI (`src/config.rs`).

Features
--------

- ✨ Intuitive TUI: navigate and edit keys/values in a compact list
- 🔎 Known keys: choose from a curated list of common mako options
- 💡 Allowed-value hints: suggested allowed values are shown while editing
- ➕ Add custom keys: pick a known key or create your own
- 🗑️ Safe delete: confirm before removing a parameter
- 💾 Save & reload feedback: quick status shown in the footer

Quick start
-----------

Prerequisites

- Rust toolchain (stable) installed via `rustup`


Build and run

There are two common ways to use the published package from crates.io:

1. Install and run the provided binary (recommended for end users):

```bash
# install the published binary
cargo install mako-tui

# then run
mako-tui
```

2. Use the repository locally or depend on it in a workspace and run the binary directly:

```bash
# from the repo root
cargo run --release
```

On first run the editor seeds the configuration with a couple helpful keys (for example `font` and `background-color`).

Controls / Keybindings
----------------------

- Up / k — move selection up
- Down / j — move selection down
- e / Enter — edit the selected value
- a — add a new key (choose from known keys or create a custom key)
- d — delete the selected key (confirmation prompt)
- q — quit the application

While editing or adding values:
- Enter — save / commit
- Esc — cancel
- Backspace — remove a character

Where to look
-------------

- `src/main.rs` — TUI layout, input handling, main loop
- `src/mako_config.rs` — typed mako config model, `known_keys()` and `allowed_values()` helpers
- `src/config.rs` — load/save logic for the key/value store used by the UI

