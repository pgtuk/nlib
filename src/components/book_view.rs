use relm4::adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use super::{clean_entry, clean_text, pick_image_file, texture_from_bytes};
use crate::models::Book;
use crate::services::covers::{self, process_image_bytes};

pub struct BookViewModal {
    book: Book,
    pending_cover_bytes: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum BookViewInput {
    Save,
    Cancel,
    PickLocalImage,
    LocalImageLoaded(Vec<u8>),
    DeleteRequested,
    DeleteConfirmed,
}

#[derive(Debug)]
pub enum BookViewOutput {
    Save(Book),
    Delete(i64),
}

#[derive(Debug)]
pub enum BookViewCmd {
    CoverProcessed(Vec<u8>),
}

#[relm4::component(pub)]
impl Component for BookViewModal {
    type Init = Book;
    type Input = BookViewInput;
    type Output = BookViewOutput;
    type CommandOutput = BookViewCmd;

    view! {
        adw::Window {
            set_title: Some(&model.book.title),
            set_modal: true,
            set_default_size: (820, 680),

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

                gtk::ScrolledWindow {
                    set_vexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        set_margin_all: 16,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 16,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 8,
                                set_valign: gtk::Align::Start,

                                #[name = "cover_picture"]
                                gtk::Picture {
                                    set_content_fit: gtk::ContentFit::Contain,
                                    set_size_request: (320, 460),
                                    add_css_class: "card",
                                },

                                gtk::Button {
                                    set_label: "Upload Cover…",
                                    set_icon_name: "insert-image-symbolic",
                                    connect_clicked => BookViewInput::PickLocalImage,
                                },
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 12,
                                set_hexpand: true,

                                adw::PreferencesGroup {
                                    set_title: "Book details",

                                    #[name = "title_entry"]
                                    adw::EntryRow {
                                        set_title: "Title",
                                    },

                                    #[name = "author_entry"]
                                    adw::EntryRow {
                                        set_title: "Author",
                                    },

                                    #[name = "cover_entry"]
                                    adw::EntryRow {
                                        set_title: "Cover image URL",
                                    },
                                },

                                adw::PreferencesGroup {
                                    set_title: "Reading status",

                                    #[name = "read_by_nika_switch"]
                                    adw::SwitchRow {
                                        set_title: "Read by Nika",
                                    },

                                    #[name = "read_by_petya_switch"]
                                    adw::SwitchRow {
                                        set_title: "Read by Petya",
                                    },
                                },

                                gtk::Label {
                                    set_label: "Description",
                                    set_xalign: 0.0,
                                    add_css_class: "heading",
                                },

                                gtk::ScrolledWindow {
                                    set_min_content_height: 180,
                                    set_vexpand: true,
                                    add_css_class: "card",

                                    #[name = "description_view"]
                                    gtk::TextView {
                                        set_wrap_mode: gtk::WrapMode::WordChar,
                                        set_top_margin: 8,
                                        set_bottom_margin: 8,
                                        set_left_margin: 8,
                                        set_right_margin: 8,
                                    },
                                },
                            },
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
        let model = BookViewModal {
            book,
            pending_cover_bytes: None,
        };

        let widgets = view_output!();

        widgets.title_entry.set_text(&model.book.title);
        widgets.author_entry.set_text(&model.book.author);
        widgets.cover_entry.set_text(&model.book.cover_url);
        widgets
            .read_by_nika_switch
            .set_active(model.book.read_by_nika);
        widgets
            .read_by_petya_switch
            .set_active(model.book.read_by_petya);
        widgets
            .description_view
            .buffer()
            .set_text(&model.book.description);

        if let Some(path) = covers::existing_cover(model.book.id) {
            widgets.cover_picture.set_filename(Some(path));
        }

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            BookViewCmd::CoverProcessed(resized) => {
                if let Some(tex) = texture_from_bytes(&resized) {
                    widgets.cover_picture.set_paintable(Some(&tex));
                    self.pending_cover_bytes = Some(resized);
                }
            }
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            BookViewInput::Cancel => {
                root.close();
            }

            BookViewInput::PickLocalImage => {
                let sender = sender.clone();
                pick_image_file(Some(root), move |bytes| {
                    sender.input(BookViewInput::LocalImageLoaded(bytes));
                });
            }

            BookViewInput::LocalImageLoaded(raw) => {
                sender.oneshot_command(async move {
                    let resized = process_image_bytes(raw).await;
                    BookViewCmd::CoverProcessed(resized)
                });
            }

            BookViewInput::Save => {
                let mut has_error = false;
                let updated = Book {
                    id: self.book.id,
                    title: clean_entry(&widgets.title_entry),
                    author: clean_entry(&widgets.author_entry),
                    description: clean_text(&widgets.description_view),
                    cover_url: clean_entry(&widgets.cover_entry),
                    read_by_nika: widgets.read_by_nika_switch.is_active(),
                    read_by_petya: widgets.read_by_petya_switch.is_active(),
                };

                widgets.title_entry.remove_css_class("error");
                widgets.author_entry.remove_css_class("error");

                if updated.title.is_empty() {
                    widgets.title_entry.add_css_class("error");
                    has_error = true;
                }
                if updated.author.is_empty() {
                    widgets.author_entry.add_css_class("error");
                    has_error = true;
                }
                if has_error {
                    return;
                }

                if let Some(bytes) = self.pending_cover_bytes.take() {
                    let _ = covers::save_cover_bytes(updated.id, &bytes);
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
