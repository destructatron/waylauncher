use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::DesktopEntryObject)]
pub struct DesktopEntryObject {
    #[property(get, set)]
    name: RefCell<String>,
    #[property(get, set)]
    generic_name: RefCell<String>,
    #[property(get, set)]
    comment: RefCell<String>,
    #[property(get, set)]
    icon: RefCell<String>,
    #[property(get, set)]
    exec: RefCell<String>,
    #[property(get, set)]
    desktop_file_path: RefCell<String>,
    #[property(get, set)]
    categories: RefCell<String>,
    #[property(get, set)]
    terminal: RefCell<bool>,
    #[property(get, set)]
    search_string: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for DesktopEntryObject {
    const NAME: &'static str = "WaylauncherDesktopEntry";
    type Type = super::DesktopEntryObject;
    type ParentType = glib::Object;
}

#[glib::derived_properties]
impl ObjectImpl for DesktopEntryObject {}
