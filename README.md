# Waylauncher

A lightweight GTK4 application launcher for Wayland. Designed to be fast, keyboard-driven, and fully accessible to screen reader users.

## Features

- Searches all XDG application directories for `.desktop` files
- Desktop icon mode (`--desktop-mode`) shows entries from `~/Desktop`
- Real-time search filtering across app name, description, and categories
- Wayland layer shell overlay with exclusive keyboard grab
- Full accessibility: labeled widgets, keyboard navigation, AT-SPI announcements via Orca

## Dependencies

### Build

- Rust 1.70+
- GTK 4.14+ development libraries
- gtk4-layer-shell development libraries

#### Fedora

```sh
sudo dnf install gtk4-devel gtk4-layer-shell-devel
```

#### Arch Linux

```sh
sudo pacman -S gtk4 gtk4-layer-shell
```

#### Ubuntu/Debian

```sh
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev
```

### Runtime

- A Wayland compositor that supports the layer shell protocol (sway, Hyprland, river, etc.)
- Falls back to a normal window on X11 (no overlay positioning)

## Install

```sh
cargo install --path .
```

Or to build without installing:

```sh
cargo build --release
```

The binary will be at `target/release/waylauncher`.

## Usage

```sh
# Launch the application launcher
waylauncher

# Launch in desktop icon mode (shows ~/Desktop entries)
waylauncher --desktop-mode
```

### Keyboard

| Key | Action |
|-----|--------|
| Any character | Filters the application list |
| Arrow keys | Navigate the list |
| Enter | Launch the selected application |
| Escape | Close the launcher |

### Environment Variables

- `RUST_LOG` — controls log output (e.g. `RUST_LOG=debug waylauncher`)
- `TERMINAL` — terminal emulator for apps with `Terminal=true` (falls back to `xdg-terminal-exec`)

## License

[MIT](LICENSE)
