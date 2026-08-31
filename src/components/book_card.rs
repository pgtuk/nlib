use relm4::factory::FactoryComponent;
use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::models::Book;

pub struct BookCard {
    book: Book,
}

#[relm4::factory(pub)]
impl FactoryComponent for BookCard {
    type Init = Book;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 6,
            add_css_class: "card",

            gtk::Picture {
                set_filename: self.book.cover_path.clone(),
                set_content_fit: gtk::ContentFit::Contain,
                set_height_request: 220,
            },

            gtk::Label {
                set_label: &self.book.title,
                add_css_class: "heading",
                set_xalign: 0.0,
                set_wrap: true,
                set_max_width_chars: 20,
                set_lines: 2,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
            },

            gtk::Label {
                set_label: &self.book.author,
                add_css_class: "dim-label",
                set_xalign: 0.0,
            }
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::prelude::FactorySender<Self>,
    ) -> Self {
        BookCard { book: init }
    }
}
