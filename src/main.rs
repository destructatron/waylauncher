mod config;
mod desktop_entry;
mod launcher;
mod scanner;
mod window;

use clap::Parser;
use gtk::prelude::*;

use config::Config;
use window::WaylauncherWindow;

fn main() {
    env_logger::init();

    let config = Config::parse();

    let app = gtk::Application::builder()
        .application_id("com.github.waylauncher")
        .build();

    app.connect_activate(move |app| {
        let window = WaylauncherWindow::new(app, config.desktop_mode);
        window.present();
    });

    app.run_with_args::<String>(&[]);
}
