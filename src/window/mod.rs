mod imp;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{AccessibleAnnouncementPriority, Application};

use crate::desktop_entry::DesktopEntryObject;

glib::wrapper! {
    pub struct WaylauncherWindow(ObjectSubclass<imp::WaylauncherWindowInner>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl WaylauncherWindow {
    pub fn new(app: &Application, desktop_mode: bool) -> Self {
        let window: Self = glib::Object::builder().property("application", app).build();

        // Set mode before populating
        window.imp().desktop_mode.set(desktop_mode);

        // Set title based on mode. Layer-shell surfaces don't get the default
        // Window role + name wiring that xdg-shell toplevels receive, so Orca's
        // "speak title" comes up empty. Set the role and label explicitly.
        let title = if desktop_mode {
            "Desktop"
        } else {
            "Waylauncher"
        };
        window.set_title(Some(title));
        window.set_accessible_role(gtk::AccessibleRole::Window);
        window.update_property(&[gtk::accessible::Property::Label(title)]);

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

        // Announce the window to screen readers. Layer-shell overlays don't
        // trigger a standard window-activate event, and `announce()` on the
        // window itself is unreliable before it's mapped, so fire on the
        // focused search entry after a short delay.
        let entry = window.imp().search_entry.clone();
        let announce_msg = title.to_string();
        glib::timeout_add_local_once(std::time::Duration::from_millis(50), move || {
            entry.announce(&announce_msg, AccessibleAnnouncementPriority::High);
        });

        window
    }
}
