mod command_window;
mod config;
mod desktop_entry;
mod launcher;
mod scanner;
mod window;

use clap::Parser;
use gtk::prelude::*;

use command_window::CommandWindow;
use config::Config;
use window::WaylauncherWindow;

fn main() {
    env_logger::init();

    let config = Config::parse();

    let app = gtk::Application::builder()
        .application_id("com.github.waylauncher")
        .build();

    app.connect_activate(move |app| {
        if config.command {
            let window = CommandWindow::new(app);
            window.present();
        } else {
            let window = WaylauncherWindow::new(app, config.desktop_mode);
            window.present();
        }
    });

    app.run_with_args::<String>(&[]);
}
