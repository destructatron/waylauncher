mod imp;

use gtk::glib;
use gtk::prelude::*;
use gtk::Application;

glib::wrapper! {
    pub struct CommandWindow(ObjectSubclass<imp::CommandWindowInner>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl CommandWindow {
    pub fn new(app: &Application) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .build();

        window.set_title(Some("Run Command"));

        window
    }
}
