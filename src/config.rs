use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "waylauncher", about = "GTK4 Wayland application launcher")]
pub struct Config {
    /// Show entries from ~/Desktop instead of XDG application directories
    #[arg(short, long)]
    pub desktop_mode: bool,
}
