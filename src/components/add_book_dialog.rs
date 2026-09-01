use relm4::adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use super::{clean_entry, clean_text, pick_image_file, texture_from_bytes};
use crate::models::BookDraft;
use crate::services::covers::{fetch_and_resize, process_image_bytes};
use crate::services::google_books::search_google_books;
use crate::services::openlibrary::search_openlibrary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchProvider {
    #[default]
    OpenLibrary,
    GoogleBooks,
}

#[derive(Debug)]
pub enum AddBookDialogMsg {
    Save,
    Cancel,
    Search,
    SetProvider(SearchProvider),
    SelectResult(usize),
    CoverUrlChanged(String),
    PickLocalImage,
    LocalImageLoaded(Vec<u8>),
    ClearSearchResults,
}

#[derive(Debug)]
pub enum AddBookDialogOutput {
    Save(BookDraft),
}

#[derive(Debug)]
pub enum DialogCmd {
    SearchResults(Vec<BookDraft>),
    CoverLoaded(Vec<u8>),
    Error(String),
}

#[derive(Default)]
pub struct AddBookDialog {
    provider: SearchProvider,
    results: Vec<BookDraft>,
    cover_bytes: Vec<u8>,
}

#[relm4::component(pub)]
impl Component for AddBookDialog {
    type Init = ();
    type Input = AddBookDialogMsg;
    type Output = AddBookDialogOutput;
    type CommandOutput = DialogCmd;

    view! {
        adw::Window {
            set_title: Some("Add New Book"),
            set_modal: true,
            set_hide_on_close: true,
            set_default_size: (820, 680),

            connect_close_request[sender] => move |_| {
                sender.input(AddBookDialogMsg::Cancel);
                gtk::glib::Propagation::Proceed
            },

            #[name = "toast_overlay"]
            adw::ToastOverlay {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::HeaderBar {
                        set_show_start_title_buttons: false,
                        set_show_end_title_buttons: false,

                        pack_start = &gtk::Button {
                            set_label: "Cancel",
                            connect_clicked => AddBookDialogMsg::Cancel,
                        },

                        pack_end = &gtk::Button {
                            set_label: "Save Book",
                            add_css_class: "suggested-action",
                            connect_clicked => AddBookDialogMsg::Save,
                        },
                    },

                    gtk::ScrolledWindow {
                        set_vexpand: true,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 16,
                            set_margin_all: 16,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 8,

                                gtk::Box {
                                    set_spacing: 8,
                                    set_valign: gtk::Align::Center,

                                    gtk::Label {
                                        set_label: "Search book on",
                                        add_css_class: "heading",
                                        set_xalign: 0.0,
                                    },

                                    gtk::Box {
                                        set_halign: gtk::Align::Start,
                                        add_css_class: "linked",

                                        #[name = "openlibrary_btn"]
                                        gtk::ToggleButton {
                                            set_label: "OpenLibrary",
                                            set_active: true,
                                            connect_toggled[sender] => move |btn| {
                                                if btn.is_active() {
                                                    sender.input(AddBookDialogMsg::SetProvider(SearchProvider::OpenLibrary));
                                                }
                                            },
                                        },

                                        #[name = "google_books_btn"]
                                        gtk::ToggleButton {
                                            set_label: "Google Books",
                                            set_group: Some(&openlibrary_btn),
                                            connect_toggled[sender] => move |btn| {
                                                if btn.is_active() {
                                                    sender.input(AddBookDialogMsg::SetProvider(SearchProvider::GoogleBooks));
                                                }
                                            },
                                        },
                                    },
                                },

                                gtk::Box {
                                    set_spacing: 6,

                                    #[name = "search_entry"]
                                    gtk::SearchEntry {
                                        set_hexpand: true,
                                        set_placeholder_text: Some("Title, author, or ISBN..."),
                                        connect_activate => AddBookDialogMsg::Search,
                                        connect_search_changed => AddBookDialogMsg::ClearSearchResults,
                                    },

                                    #[name = "search_button"]
                                    gtk::Button {
                                        set_label: "Search",
                                        connect_clicked => AddBookDialogMsg::Search,
                                    },
                                },
                            },

                            #[name = "results_scroll"]
                            gtk::ScrolledWindow {
                                set_min_content_height: 120,
                                set_visible: false,

                                #[name = "results_list"]
                                gtk::ListBox {
                                    add_css_class: "boxed-list",
                                    connect_row_activated[sender] => move |_, row| {
                                        sender.input(AddBookDialogMsg::SelectResult(row.index() as usize));
                                    },
                                },
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 16,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 8,
                                    set_valign: gtk::Align::Start,

                                    #[name = "cover_placeholder"]
                                    gtk::Frame {
                                        set_size_request: (320, 460),
                                        add_css_class: "cover-placeholder",

                                        #[name = "cover_placeholder_label"]
                                        gtk::Label {
                                            set_label: "Cover preview",
                                            add_css_class: "dim-label",
                                            set_justify: gtk::Justification::Center,
                                            set_halign: gtk::Align::Center,
                                            set_valign: gtk::Align::Center,
                                            set_wrap: true,
                                            set_max_width_chars: 20,
                                        },
                                    },

                                    #[name = "cover_preview"]
                                    gtk::Picture {
                                        set_visible: false,
                                        set_content_fit: gtk::ContentFit::Contain,
                                        set_size_request: (320, 460),
                                        add_css_class: "card",
                                    },

                                    gtk::Button {
                                        set_label: "Upload Cover…",
                                        set_icon_name: "insert-image-symbolic",
                                        connect_clicked => AddBookDialogMsg::PickLocalImage,
                                    },
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 12,
                                    set_hexpand: true,

                                    adw::PreferencesGroup {
                                        set_title: "Book details",

                                        #[name = "title_entry"]
                                        adw::EntryRow { set_title: "Title" },

                                        #[name = "author_entry"]
                                        adw::EntryRow { set_title: "Author" },

                                        #[name = "cover_entry"]
                                        adw::EntryRow {
                                            set_title: "Cover image URL (optional)",
                                            connect_changed[sender] => move |entry| {
                                                sender.input(AddBookDialogMsg::CoverUrlChanged(entry.text().to_string()));
                                            },
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
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AddBookDialog::default();
        let widgets = view_output!();
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
            DialogCmd::SearchResults(results) => {
                widgets.search_button.set_sensitive(true);
                widgets.search_button.set_label("Search");

                clear_results_ui(&mut self.results, widgets);
                self.results = results;

                for draft in &self.results {
                    let row = adw::ActionRow::builder()
                        .title(&draft.title)
                        .subtitle(&draft.author)
                        .build();

                    if draft.cover_url.is_empty() {
                        let badge = gtk::Label::builder()
                            .label("No cover")
                            .css_classes(["dim-label"])
                            .build();
                        row.add_suffix(&badge);
                    } else {
                        let badge = gtk::Label::builder()
                            .label("📷")
                            .tooltip_text("Cover available")
                            .css_classes(["dim-label"])
                            .build();
                        row.add_suffix(&badge);
                    }

                    row.set_activatable(true);
                    widgets.results_list.append(&row);
                }
                widgets.results_scroll.set_visible(!self.results.is_empty());
            }

            DialogCmd::CoverLoaded(resized) => {
                set_cover_preview(&mut self.cover_bytes, widgets, resized);
            }

            DialogCmd::Error(e) => {
                widgets.search_button.set_sensitive(true);
                widgets.search_button.set_label("Search");
                let toast = adw::Toast::new(&e);
                toast.set_timeout(5);
                widgets.toast_overlay.add_toast(toast);
            }
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            AddBookDialogMsg::Cancel => {
                clear_form(&mut self.results, &mut self.cover_bytes, widgets);
                root.set_visible(false);
            }

            AddBookDialogMsg::SetProvider(p) => {
                self.provider = p;
            }

            AddBookDialogMsg::PickLocalImage => {
                let sender = sender.clone();
                pick_image_file(Some(root), move |bytes| {
                    sender.input(AddBookDialogMsg::LocalImageLoaded(bytes));
                });
            }

            AddBookDialogMsg::LocalImageLoaded(raw) => {
                sender.oneshot_command(async move {
                    let resized = process_image_bytes(raw).await;
                    DialogCmd::CoverLoaded(resized)
                });
            }

            AddBookDialogMsg::Save => {
                let mut has_error = false;
                let draft = BookDraft {
                    title: clean_entry(&widgets.title_entry),
                    author: clean_entry(&widgets.author_entry),
                    description: clean_text(&widgets.description_view),
                    cover_url: clean_entry(&widgets.cover_entry),
                    cover_bytes: if self.cover_bytes.is_empty() {
                        None
                    } else {
                        Some(self.cover_bytes.clone())
                    },
                    read_by_nika: widgets.read_by_nika_switch.is_active(),
                    read_by_petya: widgets.read_by_petya_switch.is_active(),
                };

                widgets.title_entry.remove_css_class("error");
                widgets.author_entry.remove_css_class("error");

                if draft.title.is_empty() {
                    widgets.title_entry.add_css_class("error");
                    has_error = true;
                }
                if draft.author.is_empty() {
                    widgets.author_entry.add_css_class("error");
                    has_error = true;
                }
                if has_error {
                    return;
                }

                let _ = sender.output(AddBookDialogOutput::Save(draft));
                clear_form(&mut self.results, &mut self.cover_bytes, widgets);
                root.set_visible(false);
            }

            AddBookDialogMsg::Search => {
                let q = widgets.search_entry.text().trim().to_string();
                if q.is_empty() {
                    return;
                }
                widgets.search_button.set_sensitive(false);
                widgets.search_button.set_label("Searching...");

                let provider = self.provider;
                sender.oneshot_command(async move {
                    let res = match provider {
                        SearchProvider::OpenLibrary => {
                            search_openlibrary(&q).await.map_err(|e| e.to_string())
                        }
                        SearchProvider::GoogleBooks => {
                            search_google_books(&q).await.map_err(|e| e.to_string())
                        }
                    };
                    match res {
                        Ok(books) => DialogCmd::SearchResults(books),
                        Err(err) => DialogCmd::Error(err),
                    }
                });
            }

            AddBookDialogMsg::SelectResult(i) => {
                let Some(d) = self.results.get(i) else {
                    return;
                };

                widgets.title_entry.set_text(&d.title);
                widgets.author_entry.set_text(&d.author);
                widgets.cover_entry.set_text(&d.cover_url);
                widgets.description_view.buffer().set_text(&d.description);
                widgets.results_scroll.set_visible(false);

                // Trigger cover fetch or display "No cover available"
                sender.input(AddBookDialogMsg::CoverUrlChanged(d.cover_url.clone()));
            }

            AddBookDialogMsg::CoverUrlChanged(url) => {
                let url = url.trim().to_string();
                if url.is_empty() {
                    show_cover_placeholder(
                        &mut self.cover_bytes,
                        widgets,
                        Some("No cover image\navailable"),
                    );
                    return;
                }
                sender.oneshot_command(async move {
                    match fetch_and_resize(&url).await {
                        Ok(resized) => DialogCmd::CoverLoaded(resized),
                        Err(e) => DialogCmd::Error(format!("Failed to fetch cover: {e}")),
                    }
                });
            }

            AddBookDialogMsg::ClearSearchResults => {
                clear_results_ui(&mut self.results, widgets);
            }
        }
    }
}

fn clear_form(
    results: &mut Vec<BookDraft>,
    cover_bytes: &mut Vec<u8>,
    widgets: &mut AddBookDialogWidgets,
) {
    widgets.title_entry.set_text("");
    widgets.author_entry.set_text("");
    widgets.cover_entry.set_text("");
    widgets.read_by_nika_switch.set_active(false);
    widgets.read_by_petya_switch.set_active(false);
    widgets.description_view.buffer().set_text("");
    widgets.search_entry.set_text("");
    clear_results_ui(results, widgets);
    show_cover_placeholder(cover_bytes, widgets, Some("Cover preview"));
}

fn clear_results_ui(results: &mut Vec<BookDraft>, widgets: &mut AddBookDialogWidgets) {
    results.clear();
    while let Some(child) = widgets.results_list.first_child() {
        widgets.results_list.remove(&child);
    }
    widgets.results_scroll.set_visible(false);
}

fn show_cover_placeholder(
    cover_bytes: &mut Vec<u8>,
    widgets: &mut AddBookDialogWidgets,
    message: Option<&str>,
) {
    widgets
        .cover_preview
        .set_paintable(gtk::gdk::Paintable::NONE);
    widgets.cover_preview.set_visible(false);
    widgets
        .cover_placeholder_label
        .set_label(message.unwrap_or("Cover preview"));
    widgets.cover_placeholder.set_visible(true);
    cover_bytes.clear();
}

fn set_cover_preview(
    cover_bytes: &mut Vec<u8>,
    widgets: &mut AddBookDialogWidgets,
    resized_bytes: Vec<u8>,
) {
    if let Some(tex) = texture_from_bytes(&resized_bytes) {
        widgets.cover_preview.set_paintable(Some(&tex));
        widgets.cover_placeholder.set_visible(false);
        widgets.cover_preview.set_visible(true);
        *cover_bytes = resized_bytes;
    }
}
