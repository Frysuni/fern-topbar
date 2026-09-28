use crate::backend::notifications::Notification;
use crate::ui::icon_names;
use relm4::gtk;
use relm4::gtk::gio;
use relm4::gtk::gio::prelude::*;
use std::path::Path;

pub fn resolve(item: &Notification) -> gio::Icon {
    file_icon(&item.icon)
        .or_else(|| themed_icon(&item.icon))
        .or_else(|| item.desktop_entry.as_deref().and_then(desktop_icon))
        .unwrap_or_else(|| gio::ThemedIcon::new(icon_names::NOTIFICATIONS).upcast())
}

fn file_icon(value: &str) -> Option<gio::Icon> {
    let file = if value.starts_with("file://") {
        gio::File::for_uri(value)
    } else if Path::new(value).is_absolute() {
        gio::File::for_path(value)
    } else {
        return None;
    };

    file.query_exists(None::<&gio::Cancellable>)
        .then(|| gio::FileIcon::new(&file).upcast())
}

fn themed_icon(name: &str) -> Option<gio::Icon> {
    if name.is_empty() {
        return None;
    }

    let display = gtk::gdk::Display::default()?;

    if !gtk::IconTheme::for_display(&display).has_icon(name) {
        return None;
    }

    Some(gio::ThemedIcon::new(name).upcast())
}

fn desktop_icon(desktop_entry: &str) -> Option<gio::Icon> {
    let ids = desktop_entry_ids(desktop_entry);

    gio::AppInfo::all().into_iter().find_map(|app| {
        app.id()
            .is_some_and(|id| ids.iter().any(|candidate| id == candidate.as_str()))
            .then(|| app.icon())
            .flatten()
    })
}

fn desktop_entry_ids(desktop_entry: &str) -> Vec<String> {
    vec![desktop_entry.to_owned(), format!("{desktop_entry}.desktop")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_accepts_ids_with_desktop_in_the_name() {
        assert_eq!(
            desktop_entry_ids("org.telegram.desktop"),
            ["org.telegram.desktop", "org.telegram.desktop.desktop"]
        );
    }
}
