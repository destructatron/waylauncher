mod imp;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::Application;

use crate::desktop_entry::DesktopEntryObject;

glib::wrapper! {
    pub struct WaylauncherWindow(ObjectSubclass<imp::WaylauncherWindowInner>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl WaylauncherWindow {
    pub fn new(app: &Application, desktop_mode: bool) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .build();

        // Set mode before populating
        window.imp().desktop_mode.set(desktop_mode);

        // Set title based on mode
        if desktop_mode {
            window.set_title(Some("Desktop"));
        } else {
            window.set_title(Some("Waylauncher"));
        }

        // Update search placeholder and a11y based on mode
        if desktop_mode {
            window
                .imp()
                .search_entry
                .set_placeholder_text(Some("Search desktop entries..."));
            window
                .imp()
                .list_view
                .update_property(&[gtk::accessible::Property::Label("Desktop entries")]);
        }

        // Populate entries
        let entries: Vec<DesktopEntryObject> = if desktop_mode {
            crate::scanner::scan_desktop_directory()
        } else {
            crate::scanner::scan_applications()
        };

        let store = &window.imp().store;
        for entry in entries {
            store.append(&entry);
        }

        window
    }
}
