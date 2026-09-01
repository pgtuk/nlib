use relm4::factory::FactoryComponent;
use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::models::Book;
use crate::services::covers;

pub struct BookCard {
    book: Book,
}

#[derive(Debug)]
pub enum BookCardInput {
    Clicked,
}

#[derive(Debug)]
pub enum BookCardOutput {
    Open(Book),
}

#[relm4::factory(pub)]
impl FactoryComponent for BookCard {
    type Init = Book;
    type Input = BookCardInput;
    type Output = BookCardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Button {
            add_css_class: "book-card",
            set_size_request: (170, -1),
            connect_clicked => BookCardInput::Clicked,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 6,

                gtk::Picture {
                    #[watch]
                    set_filename: covers::existing_cover(self.book.id),
                    set_content_fit: gtk::ContentFit::Contain,
                    set_height_request: 220,
                    set_width_request: 150,
                },

                gtk::Label {
                    set_label: &self.book.title,
                    add_css_class: "heading",
                    set_xalign: 0.0,
                    set_wrap: true,
                    set_max_width_chars: 18,
                    set_lines: 2,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                },

                gtk::Label {
                    set_label: &self.book.author,
                    add_css_class: "dim-label",
                    set_xalign: 0.0,
                    set_wrap: true,
                    set_max_width_chars: 18,
                    set_lines: 1,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_visible: self.book.read_by_nika || self.book.read_by_petya,

                    gtk::Label {
                        set_label: "Nika",
                        add_css_class: "badge-nika",
                        set_visible: self.book.read_by_nika,
                    },

                    gtk::Label {
                        set_label: "Petya",
                        add_css_class: "badge-petya",
                        set_visible: self.book.read_by_petya,
                    },
                },
            }
        }
    }

    fn init_model(
        book: Self::Init,
        _index: &Self::Index,
        sender: relm4::prelude::FactorySender<Self>,
    ) -> Self {
        if covers::existing_cover(book.id).is_none() && !book.cover_url.is_empty() {
            let (id, url) = (book.id, book.cover_url.clone());
            sender.oneshot_command(async move {
                covers::download_cover(&url, id).await.ok();
            });
        }
        BookCard { book }
    }

    fn update(&mut self, _message: Self::Input, sender: relm4::prelude::FactorySender<Self>) {
        let _ = sender.output(BookCardOutput::Open(self.book.clone()));
    }
}
