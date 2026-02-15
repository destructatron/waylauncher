use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    gio, AccessibleAnnouncementPriority, CustomFilter, FilterChange, FilterListModel, Label,
    ListView, ScrolledWindow, SearchEntry, SignalListItemFactory, SingleSelection,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::desktop_entry::DesktopEntryObject;

pub struct WaylauncherWindowInner {
    pub search_entry: SearchEntry,
    pub list_view: ListView,
    pub selection: SingleSelection,
    pub filter: CustomFilter,
    pub store: gio::ListStore,
    pub query: Rc<RefCell<String>>,
    pub desktop_mode: Cell<bool>,
}

impl Default for WaylauncherWindowInner {
    fn default() -> Self {
        let store = gio::ListStore::new::<DesktopEntryObject>();
        let filter = CustomFilter::new(|_| true);
        let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));
        let selection = SingleSelection::new(Some(filter_model));
        selection.set_autoselect(true);

        let factory = SignalListItemFactory::new();
        let list_view = ListView::new(Some(selection.clone()), Some(factory.clone()));

        setup_factory(&factory);

        Self {
            search_entry: SearchEntry::new(),
            list_view,
            selection,
            filter,
            store,
            query: Rc::new(RefCell::new(String::new())),
            desktop_mode: Cell::new(false),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for WaylauncherWindowInner {
    const NAME: &'static str = "WaylauncherWindow";
    type Type = super::WaylauncherWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for WaylauncherWindowInner {
    fn constructed(&self) {
        self.parent_constructed();

        let window = self.obj();

        // Layer shell setup
        if gtk4_layer_shell::is_supported() {
            window.init_layer_shell();
            window.set_layer(Layer::Overlay);
            window.set_keyboard_mode(KeyboardMode::Exclusive);
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Top, 100);
            window.set_namespace(Some("waylauncher"));
        }

        window.set_default_size(600, 500);

        // Build widget tree
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        // Search entry
        self.search_entry
            .set_placeholder_text(Some("Search applications..."));
        self.search_entry.set_hexpand(true);

        // Accessibility for search entry
        self.search_entry.update_property(&[
            gtk::accessible::Property::Label("Search applications"),
            gtk::accessible::Property::Description(
                "Type to filter applications. Use arrow keys to navigate the list, Enter to launch.",
            ),
        ]);

        // Scrolled window with list view
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();
        scrolled.set_child(Some(&self.list_view));

        // Accessibility for list view
        let list_label = if self.desktop_mode.get() {
            "Desktop entries"
        } else {
            "Applications"
        };
        self.list_view
            .update_property(&[gtk::accessible::Property::Label(list_label)]);

        vbox.append(&self.search_entry);
        vbox.append(&scrolled);

        window.set_child(Some(&vbox));

        // Key capture: typing anywhere goes to search entry
        self.search_entry
            .set_key_capture_widget(Some(window.as_ref()));

        // Connect filter logic
        self.connect_filter();

        // Connect activation
        self.connect_activate();

        // Connect Escape to close
        self.connect_escape();

        // Focus search entry
        self.search_entry.grab_focus();
    }
}

impl WidgetImpl for WaylauncherWindowInner {}
impl WindowImpl for WaylauncherWindowInner {}
impl ApplicationWindowImpl for WaylauncherWindowInner {}

impl WaylauncherWindowInner {
    fn connect_filter(&self) {
        let filter = self.filter.clone();
        let query = self.query.clone();
        let selection = self.selection.clone();

        self.search_entry.connect_search_changed(move |entry| {
            let text = entry.text().to_string().to_lowercase();
            *query.borrow_mut() = text;
            filter.changed(FilterChange::Different);
            selection.set_selected(0);

            // Announce result count for screen readers via AT-SPI
            let count = selection.model().map(|m| m.n_items()).unwrap_or(0);
            let announcement = format!("{} applications found", count);
            entry.announce(&announcement, AccessibleAnnouncementPriority::Medium);
        });

        let query = self.query.clone();
        self.filter.set_filter_func(move |obj| {
            let q = query.borrow();
            if q.is_empty() {
                return true;
            }
            let entry = obj.downcast_ref::<DesktopEntryObject>().unwrap();
            entry.search_string().contains(q.as_str())
        });
    }

    fn connect_activate(&self) {
        let selection = self.selection.clone();

        // ListView row activation
        let sel = selection.clone();
        self.list_view.connect_activate(move |list_view, _pos| {
            if let Some(item) = sel.selected_item() {
                let entry = item.downcast_ref::<DesktopEntryObject>().unwrap();
                crate::launcher::launch(&entry.exec(), entry.terminal());
                if let Some(window) = list_view.root().and_downcast::<gtk::Window>() {
                    window.close();
                }
            }
        });

        // SearchEntry Enter key launches selected item
        let sel = selection.clone();
        self.search_entry.connect_activate(move |entry| {
            if let Some(item) = sel.selected_item() {
                let de = item.downcast_ref::<DesktopEntryObject>().unwrap();
                crate::launcher::launch(&de.exec(), de.terminal());
                if let Some(window) = entry.root().and_downcast::<gtk::Window>() {
                    window.close();
                }
            }
        });
    }

    fn connect_escape(&self) {
        // SearchEntry captures all keys via set_key_capture_widget, so Escape
        // triggers its built-in "stop-search" signal rather than reaching a
        // window-level EventControllerKey. Connect to that signal instead.
        self.search_entry.connect_stop_search(|entry| {
            if let Some(window) = entry.root().and_downcast::<gtk::Window>() {
                window.close();
            }
        });
    }
}

fn setup_factory(factory: &SignalListItemFactory) {
    factory.connect_setup(|_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();

        let icon = gtk::Image::builder()
            .pixel_size(32)
            .margin_end(8)
            .build();

        let name_label = Label::builder()
            .xalign(0.0)
            .css_classes(["heading"])
            .build();

        let comment_label = Label::builder()
            .xalign(0.0)
            .css_classes(["dim-label"])
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();

        let text_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(2)
            .build();
        text_box.append(&name_label);
        text_box.append(&comment_label);

        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(4)
            .margin_end(4)
            .build();
        row.append(&icon);
        row.append(&text_box);

        // Bind properties via expressions
        let item_expr = gtk::ConstantExpression::new(item);
        let entry_expr =
            gtk::PropertyExpression::new(gtk::ListItem::static_type(), Some(&item_expr), "item");

        // Name
        let name_expr = gtk::PropertyExpression::new(
            DesktopEntryObject::static_type(),
            Some(&entry_expr),
            "name",
        );
        name_expr.bind(&name_label, "label", gtk::Widget::NONE);

        // Comment
        let comment_expr = gtk::PropertyExpression::new(
            DesktopEntryObject::static_type(),
            Some(&entry_expr),
            "comment",
        );
        comment_expr.bind(&comment_label, "label", gtk::Widget::NONE);

        // Icon
        let icon_expr = gtk::PropertyExpression::new(
            DesktopEntryObject::static_type(),
            Some(&entry_expr),
            "icon",
        );
        icon_expr.bind(&icon, "icon-name", gtk::Widget::NONE);

        // Use tooltip for accessible name on the row
        name_expr.bind(&row, "tooltip-text", gtk::Widget::NONE);

        item.set_child(Some(&row));
    });

    // Set accessible label when items are bound to data
    factory.connect_bind(|_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(entry) = item.item().and_downcast::<DesktopEntryObject>() {
            if let Some(row) = item.child() {
                row.update_property(&[gtk::accessible::Property::Label(&entry.name())]);
            }
        }
    });
}
