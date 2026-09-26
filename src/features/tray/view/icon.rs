use super::super::backend::Icon;
use relm4::gtk;
use relm4::gtk::prelude::*;

pub trait TrayImageExt {
    fn set_tray_icon(&self, icon: &Icon, icon_theme_path: Option<&str>, logical_size: i32);
}

impl TrayImageExt for gtk::Image {
    fn set_tray_icon(&self, icon: &Icon, icon_theme_path: Option<&str>, logical_size: i32) {
        let display = self.display();

        if let Some(path) = icon_theme_path {
            gtk::IconTheme::for_display(&display).add_search_path(path);
        }

        update(self, icon, &display, logical_size, self.scale_factor());
    }
}

pub fn update(
    image: &gtk::Image,
    icon: &Icon,
    display: &gtk::gdk::Display,
    logical_size: i32,
    scale_factor: i32,
) {
    image.clear();
    image.set_pixel_size(logical_size);

    if icon.name.starts_with('/') {
        if let Ok(texture) = gtk::gdk::Texture::from_filename(&icon.name) {
            image.set_paintable(Some(&texture));
            return;
        }
    } else if !icon.name.is_empty() && gtk::IconTheme::for_display(display).has_icon(&icon.name) {
        image.set_icon_name(Some(&icon.name));
        return;
    }

    let target_size = logical_size.saturating_mul(scale_factor.max(1));

    if let Some(pixmap) = icon.best_pixmap(target_size) {
        let mut rgba = Vec::with_capacity(pixmap.argb.len());

        for &[alpha, red, green, blue] in pixmap.argb.as_chunks::<4>().0 {
            rgba.extend_from_slice(&[red, green, blue, alpha]);
        }

        let bytes = gtk::glib::Bytes::from_owned(rgba);
        let texture = gtk::gdk::MemoryTexture::new(
            pixmap.width,
            pixmap.height,
            gtk::gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            (pixmap.width as usize) * 4,
        );

        image.set_paintable(Some(&texture));
    } else {
        image.set_icon_name(Some("image-missing-symbolic"));
    }
}
