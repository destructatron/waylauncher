use std::cell::{Cell, RefCell};
use std::path::Path;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{AccessibleAnnouncementPriority, Entry, Label, Revealer, RevealerTransitionType};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub struct CommandWindowInner {
    pub entry: Entry,
    pub toast_label: Label,
    pub toast_revealer: Revealer,
    /// Command history entries, newest first.
    pub history: RefCell<Vec<String>>,
    /// Current position in history (0 = newest). None = not browsing history.
    pub history_index: Cell<Option<usize>>,
}

impl Default for CommandWindowInner {
    fn default() -> Self {
        let toast_label = Label::builder()
            .label("")
            .css_classes(["error"])
            .xalign(0.0)
            .build();

        let toast_revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::SlideDown)
            .transition_duration(200)
            .reveal_child(false)
            .child(&toast_label)
            .build();

        Self {
            entry: Entry::builder()
                .placeholder_text("Run a command...")
                .hexpand(true)
                .build(),
            toast_label,
            toast_revealer,
            history: RefCell::new(Vec::new()),
            history_index: Cell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CommandWindowInner {
    const NAME: &'static str = "CommandWindow";
    type Type = super::CommandWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for CommandWindowInner {
    fn constructed(&self) {
        self.parent_constructed();

        let window = self.obj();

        // Layer shell setup
        if gtk4_layer_shell::is_supported() {
            window.init_layer_shell();
            window.set_layer(Layer::Overlay);
            window.set_keyboard_mode(KeyboardMode::Exclusive);
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Top, 200);
            window.set_namespace(Some("waylauncher"));
        }

        window.set_default_size(500, -1);
        window.set_resizable(false);

        // Build widget tree
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        vbox.set_margin_top(16);
        vbox.set_margin_bottom(16);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);

        // Accessibility
        self.entry.update_property(&[
            gtk::accessible::Property::Label("Run a command"),
            gtk::accessible::Property::Description(
                "Type a shell command and press Enter to execute it. Press Tab to autocomplete.",
            ),
        ]);

        vbox.append(&self.entry);
        vbox.append(&self.toast_revealer);

        window.set_child(Some(&vbox));

        // Load waylauncher command history and pre-fill with last command
        let history = load_history();
        if let Some(last) = history.first() {
            self.entry.set_text(last);
            self.entry.set_position(last.len() as i32);
            self.history_index.set(Some(0));
        }
        *self.history.borrow_mut() = history;

        // Tab completion and history navigation on the entry
        let tab_controller = gtk::EventControllerKey::new();
        let entry_for_tab = self.entry.clone();
        let history_ref = self.history.clone();
        let history_index_ref = self.history_index.clone();
        tab_controller.connect_key_pressed(move |_controller, key, _code, _mods| {
            match key {
                k if k == gtk::gdk::Key::Tab => {
                    complete_entry(&entry_for_tab);
                    return gtk::glib::Propagation::Stop;
                }
                k if k == gtk::gdk::Key::Down => {
                    // Down = older commands
                    let history = history_ref.borrow();
                    if history.is_empty() {
                        return gtk::glib::Propagation::Stop;
                    }
                    let current = history_index_ref.get().unwrap_or(0);
                    let next = (current + 1).min(history.len() - 1);
                    if next != current {
                        history_index_ref.set(Some(next));
                        entry_for_tab.set_text(&history[next]);
                        entry_for_tab.set_position(history[next].len() as i32);
                        entry_for_tab
                            .announce(&history[next], AccessibleAnnouncementPriority::Medium);
                    }
                    return gtk::glib::Propagation::Stop;
                }
                k if k == gtk::gdk::Key::Up => {
                    // Up = newer commands; at top, move cursor to start
                    let history = history_ref.borrow();
                    let current = history_index_ref.get().unwrap_or(0);
                    if current == 0 {
                        // Already at newest — move cursor to start of line
                        entry_for_tab.set_position(0);
                    } else {
                        let next = current - 1;
                        history_index_ref.set(Some(next));
                        entry_for_tab.set_text(&history[next]);
                        entry_for_tab.set_position(history[next].len() as i32);
                        entry_for_tab
                            .announce(&history[next], AccessibleAnnouncementPriority::Medium);
                    }
                    return gtk::glib::Propagation::Stop;
                }
                _ => {}
            }
            gtk::glib::Propagation::Proceed
        });
        self.entry.add_controller(tab_controller);

        // Enter runs the command
        let toast_label = self.toast_label.clone();
        let toast_revealer = self.toast_revealer.clone();
        self.entry.connect_activate(move |entry| {
            let text = entry.text();
            let command = text.trim().to_string();
            if command.is_empty() {
                return;
            }

            match crate::launcher::launch_shell(&command) {
                Ok(()) => {
                    save_history_entry(&command);
                    if let Some(window) = entry.root().and_downcast::<gtk::Window>() {
                        window.close();
                    }
                }
                Err(msg) => {
                    show_toast(entry, &toast_label, &toast_revealer, &msg);
                }
            }
        });

        // Escape closes
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(|controller, key, _code, _mods| {
            if key == gtk::gdk::Key::Escape {
                if let Some(window) = controller
                    .widget()
                    .and_then(|w| w.root())
                    .and_then(|r| r.downcast::<gtk::Window>().ok())
                {
                    window.close();
                }
                return gtk::glib::Propagation::Stop;
            }
            gtk::glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        // Focus the entry
        self.entry.grab_focus();
    }
}

impl WidgetImpl for CommandWindowInner {}
impl WindowImpl for CommandWindowInner {}
impl ApplicationWindowImpl for CommandWindowInner {}

/// Show an error toast, announce it for screen readers, and select text for correction.
fn show_toast(entry: &Entry, toast_label: &Label, toast_revealer: &Revealer, msg: &str) {
    toast_label.set_label(msg);
    toast_revealer.set_reveal_child(true);

    entry.announce(msg, AccessibleAnnouncementPriority::High);

    let revealer = toast_revealer.clone();
    glib::timeout_add_seconds_local_once(4, move || {
        revealer.set_reveal_child(false);
    });

    entry.select_region(0, -1);
    entry.grab_focus();
}

/// Perform tab completion on the entry's current text.
/// First token: complete command names from $PATH.
/// Subsequent tokens: complete file/directory paths.
fn complete_entry(entry: &Entry) {
    let text = entry.text().to_string();
    let cursor = entry.position() as usize;

    // Work on text up to the cursor
    let before_cursor = &text[..cursor.min(text.len())];
    let after_cursor = &text[cursor.min(text.len())..];

    // Find the start of the current token (respect spaces)
    let token_start = before_cursor.rfind(' ').map(|i| i + 1).unwrap_or(0);
    let prefix = &before_cursor[token_start..];
    let is_first_token = !before_cursor[..token_start].contains(|c: char| !c.is_whitespace());

    if prefix.is_empty() {
        return;
    }

    let completion = if is_first_token {
        complete_command(prefix)
    } else {
        complete_path(prefix)
    };

    if let Some(completed) = completion {
        let new_text = format!("{}{}{}", &before_cursor[..token_start], completed, after_cursor);
        entry.set_text(&new_text);
        entry.set_position((token_start + completed.len()) as i32);

        // Announce completion for screen readers
        if completed != prefix {
            entry.announce(&completed, AccessibleAnnouncementPriority::Medium);
        }
    }
}

/// Complete a command name by scanning $PATH directories.
fn complete_command(prefix: &str) -> Option<String> {
    let path_var = std::env::var("PATH").ok()?;
    let mut matches: Vec<String> = Vec::new();

    for dir in path_var.split(':') {
        let dir_path = Path::new(dir);
        let entries = match std::fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(prefix) && !matches.contains(&name.to_string()) {
                matches.push(name.to_string());
            }
        }
    }

    if matches.is_empty() {
        return None;
    }

    matches.sort();

    if matches.len() == 1 {
        // Single match — complete fully and add a trailing space
        Some(format!("{} ", matches[0]))
    } else {
        // Multiple matches — complete to longest common prefix
        Some(longest_common_prefix(&matches))
    }
}

/// Complete a file or directory path.
fn complete_path(prefix: &str) -> Option<String> {
    let expanded = if prefix.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            home.to_string_lossy().to_string() + &prefix[1..]
        } else {
            prefix.to_string()
        }
    } else {
        prefix.to_string()
    };

    let (dir, file_prefix) = if let Some(slash_pos) = expanded.rfind('/') {
        let dir = if slash_pos == 0 {
            "/".to_string()
        } else {
            expanded[..slash_pos].to_string()
        };
        (dir, expanded[slash_pos + 1..].to_string())
    } else {
        (".".to_string(), expanded.clone())
    };

    let entries = std::fs::read_dir(&dir).ok()?;
    let mut matches: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if name.starts_with(&file_prefix) {
            // Reconstruct using the original prefix style (preserve ~)
            let full = if prefix.contains('/') {
                let prefix_dir = &prefix[..prefix.rfind('/').unwrap() + 1];
                format!("{}{}", prefix_dir, name)
            } else {
                name.clone()
            };

            // Add trailing / for directories
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            if is_dir {
                matches.push(format!("{}/", full));
            } else {
                matches.push(format!("{} ", full));
            }
        }
    }

    if matches.is_empty() {
        return None;
    }

    matches.sort();

    if matches.len() == 1 {
        Some(matches.remove(0))
    } else {
        // Strip trailing / and space for common prefix calculation, then return raw prefix
        let stripped: Vec<String> = matches
            .iter()
            .map(|m| m.trim_end_matches('/').trim_end().to_string())
            .collect();
        Some(longest_common_prefix(&stripped))
    }
}

/// Path to waylauncher's own command history file.
fn history_path() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|d| d.join("waylauncher").join("history"))
}

/// Load waylauncher command history. Returns entries newest-first.
fn load_history() -> Vec<String> {
    let path = match history_path() {
        Some(p) => p,
        None => return Vec::new(),
    };
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    // File stores oldest-first (append order), so reverse for newest-first
    contents
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .rev()
        .collect()
}

/// Append a command to waylauncher's history file.
fn save_history_entry(command: &str) {
    let path = match history_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "{}", command);
    }
}

/// Find the longest common prefix among a set of strings.
fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first = &strings[0];
    let mut len = first.len();
    for s in &strings[1..] {
        len = len.min(s.len());
        for (i, (a, b)) in first.chars().zip(s.chars()).enumerate() {
            if a != b {
                len = len.min(i);
                break;
            }
        }
    }
    first[..len].to_string()
}
