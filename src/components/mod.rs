pub mod add_book_dialog;
pub mod book_card;
pub mod book_view;

use relm4::adw::prelude::*;
use relm4::{adw, gtk};

pub fn clean_entry(input: &adw::EntryRow) -> String {
    input.text().trim().to_string()
}

pub fn clean_text(input: &gtk::TextView) -> String {
    let buffer = input.buffer();
    let s = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
    s.trim().to_string()
}

pub fn texture_from_bytes(bytes: &[u8]) -> Option<gtk::gdk::Texture> {
    let g_bytes = gtk::glib::Bytes::from(bytes);
    gtk::gdk::Texture::from_bytes(&g_bytes).ok()
}

pub fn pick_image_file<F>(parent: Option<&impl IsA<gtk::Window>>, on_selected: F)
where
    F: FnOnce(Vec<u8>) + 'static,
{
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Image files (*.png, *.jpg, *.jpeg, *.webp)"));
    filter.add_pattern("*.png");
    filter.add_pattern("*.jpg");
    filter.add_pattern("*.jpeg");
    filter.add_pattern("*.webp");

    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::builder()
        .title("Select Book Cover Image")
        .filters(&filters)
        .build();

    dialog.open(parent, gtk::gio::Cancellable::NONE, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                if let Ok(bytes) = std::fs::read(&path) {
                    on_selected(bytes);
                }
            }
        }
    });
}
