# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Optimized build
cargo check                    # Fast type-check without codegen
cargo clippy                   # Lint
cargo fmt                      # Format
cargo run                      # Run (app launcher mode)
cargo run -- --desktop-mode    # Run (~/Desktop entries mode)
```

No test suite yet. The project has no Makefile or justfile — all builds go through cargo.

## Architecture

GTK4 Wayland application launcher in Rust. Two modes: XDG application launcher (default) and desktop icon mode (`--desktop-mode`). Uses gtk4-layer-shell for Wayland overlay positioning with exclusive keyboard grab.

### GObject Two-Module Pattern

Every GObject type uses a `mod.rs` / `imp.rs` split:
- **`mod.rs`** — public wrapper type via `glib::wrapper!`, constructor, public API
- **`imp.rs`** — `ObjectSubclass` impl with struct fields, `ObjectImpl::constructed()` for setup

This applies to `desktop_entry/` (data model) and `window/` (UI).

### Data Flow

`ListStore<DesktopEntryObject>` → `FilterListModel` (CustomFilter) → `SingleSelection` → `ListView`

The filter and search callback share query state via `Rc<RefCell<String>>`. The `CustomFilter` func reads it; `SearchEntry::connect_search_changed` writes it then calls `filter.changed(FilterChange::Different)`.

Widget content is bound via `PropertyExpression` chains (not imperative updates), which handles ListView cell recycling automatically.

### Key GTK4 API Patterns

- **Accessible properties** use enum variants with values: `Property::Label("text")`, passed as `&[Property]` to `update_property()`. They are NOT GObject properties and cannot be bound with expressions — use `connect_bind` on factories instead.
- **`announce()`** (GTK 4.14, `v4_14` feature) is the correct way to make screen reader announcements. Do not toggle widget visibility as a hack.
- **`SearchEntry::set_key_capture_widget(&window)`** captures ALL keyboard input, including Escape. Handle Escape via `connect_stop_search`, not a window-level EventControllerKey.
- **`EventControllerExt::widget()`** returns `Option<Widget>` in gtk4-rs 0.10.
- **Window subclass `@implements`** must include `ConstraintTarget` (plus `Accessible`, `Buildable`, `Native`, `Root`, `ShortcutManager`).

### Layer Shell

Conditional on `gtk4_layer_shell::is_supported()` — falls back to normal window on X11. Setup (init, set_layer, set_keyboard_mode, anchors) must happen in `constructed()` before the window is realized. `set_namespace()` takes `Option<&str>`.

### Desktop Entry Scanning

`scanner.rs` uses `freedesktop-desktop-entry` crate's `Iter` + `default_paths()`. Entries are filtered (Type=Application, no NoDisplay/Hidden, must have Name+Exec), deduplicated by appid (user dirs win), and sorted case-insensitively. `launcher.rs` strips freedesktop field codes (%f, %F, %u, %U, etc.) before spawning.
