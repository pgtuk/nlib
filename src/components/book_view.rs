use relm4::adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::models::Book;
use crate::services::covers;

pub struct BookViewModal {
    book: Book,
    title_entry: adw::EntryRow,
    author_entry: adw::EntryRow,
    cover_entry: adw::EntryRow,
    read_by_nika_switch: adw::SwitchRow,
    read_by_petya_switch: adw::SwitchRow,
    description_view: gtk::TextView,
}

#[derive(Debug)]
pub enum BookViewInput {
    Save,
    Cancel,
    DeleteRequested,
    DeleteConfirmed,
}

#[derive(Debug)]
pub enum BookViewOutput {
    Save(Book),
    Delete(i64),
}

impl BookViewModal {
    fn updated_book(&self) -> Book {
        Book {
            id: self.book.id,
            title: clean_entry(&self.title_entry),
            author: clean_entry(&self.author_entry),
            description: clean_text(&self.description_view),
            cover_url: clean_entry(&self.cover_entry),
            read_by_nika: self.read_by_nika_switch.is_active(),
            read_by_petya: self.read_by_petya_switch.is_active(),
        }
    }
}

#[relm4::component(pub)]
impl Component for BookViewModal {
    type Init = Book;
    type Input = BookViewInput;
    type Output = BookViewOutput;
    type CommandOutput = ();

    view! {
        adw::Window {
            set_title: Some(&model.book.title),
            set_modal: true,
            set_default_size: (520, 640),

            connect_close_request[sender] => move |_| {
                sender.input(BookViewInput::Cancel);
                gtk::glib::Propagation::Proceed
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    set_show_start_title_buttons: false,
                    set_show_end_title_buttons: false,

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked => BookViewInput::Cancel,
                    },

                    pack_end = &gtk::Button {
                        set_label: "Save",
                        add_css_class: "suggested-action",
                        connect_clicked => BookViewInput::Save,
                    },

                    pack_end = &gtk::Button {
                        set_icon_name: "user-trash-symbolic",
                        set_tooltip_text: Some("Delete book"),
                        add_css_class: "destructive-action",
                        connect_clicked => BookViewInput::DeleteRequested,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 12,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,

                            gtk::Picture {
                                set_filename: covers::existing_cover(model.book.id),
                                set_content_fit: gtk::ContentFit::Contain,
                                set_size_request: (140, 210),
                            },
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 10,
                            set_hexpand: true,

                            adw::PreferencesGroup {
                                set_title: "Book details",

                                #[local_ref]
                                title_entry -> adw::EntryRow {
                                    set_title: "Title",
                                },
                                #[local_ref]
                                author_entry -> adw::EntryRow {
                                    set_title: "Author",
                                },
                                #[local_ref]
                                cover_entry -> adw::EntryRow {
                                    set_title: "Cover image URL",
                                },
                            },

                            adw::PreferencesGroup {
                                set_title: "Reading status",

                                #[local_ref]
                                read_by_nika_switch -> adw::SwitchRow {
                                    set_title: "Read by Nika",
                                },
                                #[local_ref]
                                read_by_petya_switch -> adw::SwitchRow {
                                    set_title: "Read by Petya",
                                },
                            },
                        },
                    },

                    gtk::Label {
                        set_label: "Description",
                        set_xalign: 0.0,
                        add_css_class: "heading",
                    },

                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        add_css_class: "card",

                        #[local_ref]
                        description_view -> gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                            set_top_margin: 8,
                            set_bottom_margin: 8,
                            set_left_margin: 8,
                            set_right_margin: 8,
                        },
                    },
                },
            }
        }
    }

    fn init(
        book: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let title_entry = adw::EntryRow::default();
        title_entry.set_text(&book.title);

        let author_entry = adw::EntryRow::default();
        author_entry.set_text(&book.author);

        let cover_entry = adw::EntryRow::default();
        cover_entry.set_text(&book.cover_url);

        let read_by_nika_switch = adw::SwitchRow::default();
        read_by_nika_switch.set_active(book.read_by_nika);

        let read_by_petya_switch = adw::SwitchRow::default();
        read_by_petya_switch.set_active(book.read_by_petya);

        let description_view = gtk::TextView::new();
        description_view.buffer().set_text(&book.description);

        let model = BookViewModal {
            book,
            title_entry,
            author_entry,
            cover_entry,
            read_by_nika_switch,
            read_by_petya_switch,
            description_view,
        };

        let title_entry = &model.title_entry;
        let author_entry = &model.author_entry;
        let cover_entry = &model.cover_entry;
        let read_by_nika_switch = &model.read_by_nika_switch;
        let read_by_petya_switch = &model.read_by_petya_switch;
        let description_view = &model.description_view;

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            BookViewInput::Cancel => {
                root.close();
            }

            BookViewInput::Save => {
                let mut has_error = false;
                let updated = self.updated_book();

                self.title_entry.remove_css_class("error");
                self.author_entry.remove_css_class("error");

                if updated.title.is_empty() {
                    self.title_entry.add_css_class("error");
                    has_error = true;
                }
                if updated.author.is_empty() {
                    self.author_entry.add_css_class("error");
                    has_error = true;
                }
                if has_error {
                    return;
                }

                let _ = sender.output(BookViewOutput::Save(updated));
                root.close();
            }

            BookViewInput::DeleteRequested => {
                let dialog = adw::AlertDialog::new(
                    Some("Delete this book?"),
                    Some(&format!(
                        "“{}” will be removed permanently.",
                        self.book.title
                    )),
                );
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("delete", "Delete");
                dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                dialog.set_close_response("cancel");
                dialog.set_default_response(Some("cancel"));

                let sender = sender.clone();
                dialog.connect_response(None, move |_, response| {
                    if response == "delete" {
                        sender.input(BookViewInput::DeleteConfirmed);
                    }
                });

                dialog.present(Some(root));
            }

            BookViewInput::DeleteConfirmed => {
                let _ = sender.output(BookViewOutput::Delete(self.book.id));
                root.close();
            }
        }
    }
}

fn clean_entry(input: &adw::EntryRow) -> String {
    input.text().trim().to_string()
}

fn clean_text(input: &gtk::TextView) -> String {
    let buffer = input.buffer();
    let s = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
    s.trim().to_string()
}
