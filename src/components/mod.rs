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
