mod imp;

use gtk::glib;
glib::wrapper! {
    pub struct DesktopEntryObject(ObjectSubclass<imp::DesktopEntryObject>);
}

impl DesktopEntryObject {
    pub fn new(
        name: &str,
        generic_name: &str,
        comment: &str,
        icon: &str,
        exec: &str,
        desktop_file_path: &str,
        categories: &str,
        terminal: bool,
    ) -> Self {
        let search_string = format!(
            "{} {} {} {}",
            name, generic_name, comment, categories
        )
        .to_lowercase();

        glib::Object::builder()
            .property("name", name)
            .property("generic-name", generic_name)
            .property("comment", comment)
            .property("icon", icon)
            .property("exec", exec)
            .property("desktop-file-path", desktop_file_path)
            .property("categories", categories)
            .property("terminal", terminal)
            .property("search-string", &search_string)
            .build()
    }
}
