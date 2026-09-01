use relm4::adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::models::BookDraft;
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
    ClearSearchResults,
}

#[derive(Debug)]
pub enum AddBookDialogOutput {
    Save(BookDraft),
    Error(String),
}

#[derive(Debug)]
pub enum DialogCmd {
    SearchResult(Result<Vec<BookDraft>, String>),
    CoverBytes(Result<Vec<u8>, reqwest::Error>),
}

pub struct AddBookDialog {
    window: adw::Window,

    title_entry: adw::EntryRow,
    author_entry: adw::EntryRow,
    cover_entry: adw::EntryRow,
    read_by_nika_switch: adw::SwitchRow,
    read_by_petya_switch: adw::SwitchRow,
    description_view: gtk::TextView,

    cover_bytes: Vec<u8>,
    cover_preview: gtk::Picture,
    cover_placeholder: gtk::Frame,
    cover_placeholder_label: gtk::Label,

    provider: SearchProvider,
    search_entry: gtk::SearchEntry,
    search_button: gtk::Button,
    openlibrary_btn: gtk::ToggleButton,
    google_books_btn: gtk::ToggleButton,
    results_list: gtk::ListBox,
    results_scroll: gtk::ScrolledWindow,
    results: Vec<BookDraft>,
}

impl AddBookDialog {
    fn draft(&self) -> BookDraft {
        BookDraft {
            title: clean_entry(&self.title_entry),
            author: clean_entry(&self.author_entry),
            description: clean_text(&self.description_view),
            cover_url: clean_entry(&self.cover_entry),
            cover_bytes: if self.cover_bytes.is_empty() {
                None
            } else {
                Some(self.cover_bytes.clone())
            },
            read_by_nika: self.read_by_nika_switch.is_active(),
            read_by_petya: self.read_by_petya_switch.is_active(),
        }
    }

    fn clear(&mut self) {
        self.title_entry.set_text("");
        self.author_entry.set_text("");
        self.cover_entry.set_text("");
        self.read_by_nika_switch.set_active(false);
        self.read_by_petya_switch.set_active(false);
        self.description_view.buffer().set_text("");

        self.search_entry.set_text("");
        self.clear_results();
    }

    fn clear_results(&mut self) {
        self.results.clear();
        while let Some(child) = self.results_list.first_child() {
            self.results_list.remove(&child);
        }
        self.results_scroll.set_visible(false);
        self.show_cover_placeholder(Some("Cover preview"));
    }

    fn show_cover_placeholder(&mut self, message: Option<&str>) {
        self.cover_preview.set_paintable(gtk::gdk::Paintable::NONE);
        self.cover_preview.set_visible(false);
        self.cover_placeholder_label
            .set_label(message.unwrap_or("Cover preview"));
        self.cover_placeholder.set_visible(true);
        self.cover_bytes.clear();
    }
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

                                    #[local_ref]
                                    openlibrary_btn -> gtk::ToggleButton {
                                        set_label: "OpenLibrary",
                                        set_active: true,
                                        connect_toggled[sender] => move |btn| {
                                            if btn.is_active() {
                                                sender.input(AddBookDialogMsg::SetProvider(SearchProvider::OpenLibrary));
                                            }
                                        },
                                    },

                                    #[local_ref]
                                    google_books_btn -> gtk::ToggleButton {
                                        set_label: "Google Books",
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

                                #[local_ref]
                                search_entry -> gtk::SearchEntry {
                                    set_hexpand: true,
                                    set_placeholder_text: Some("Title or author..."),
                                    connect_activate => AddBookDialogMsg::Search,
                                    connect_search_changed => AddBookDialogMsg::ClearSearchResults,
                                },

                                #[local_ref]
                                search_button -> gtk::Button {
                                    set_label: "Search",
                                    connect_clicked => AddBookDialogMsg::Search,
                                },
                            },
                        },

                        #[local_ref]
                        results_scroll -> gtk::ScrolledWindow {
                            set_min_content_height: 120,
                            set_visible: false,

                            #[local_ref]
                            results_list -> gtk::ListBox {
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
                                set_valign: gtk::Align::Start,

                                #[local_ref]
                                cover_placeholder -> gtk::Frame {
                                    set_size_request: (320, 460),
                                    add_css_class: "cover-placeholder",

                                    #[local_ref]
                                    cover_placeholder_label -> gtk::Label {
                                        set_label: "Cover preview",
                                        add_css_class: "dim-label",
                                        set_justify: gtk::Justification::Center,
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        set_wrap: true,
                                        set_max_width_chars: 20,
                                    },
                                },

                                #[local_ref]
                                cover_preview -> gtk::Picture {
                                    set_visible: false,
                                    set_content_fit: gtk::ContentFit::Contain,
                                    set_size_request: (320, 460),
                                    add_css_class: "card",
                                },
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 12,
                                set_hexpand: true,

                                adw::PreferencesGroup {
                                    set_title: "Book details",

                                    #[local_ref]
                                    title_entry -> adw::EntryRow { set_title: "Title" },
                                    #[local_ref]
                                    author_entry -> adw::EntryRow { set_title: "Author" },
                                    #[local_ref]
                                    cover_entry -> adw::EntryRow {
                                        set_title: "Cover image URL (optional)",
                                        connect_changed[sender] => move |entry| {
                                            sender.input(AddBookDialogMsg::CoverUrlChanged(entry.text().to_string()));
                                        },
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

                                gtk::Label {
                                    set_label: "Description",
                                    set_xalign: 0.0,
                                    add_css_class: "heading",
                                },

                                gtk::ScrolledWindow {
                                    set_min_content_height: 180,
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
                        },
                    },
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let openlibrary_btn = gtk::ToggleButton::with_label("OpenLibrary");
        let google_books_btn = gtk::ToggleButton::with_label("Google Books");
        google_books_btn.set_group(Some(&openlibrary_btn));

        let cover_placeholder_label = gtk::Label::new(Some("Cover preview"));

        let model = AddBookDialog {
            window: root.clone(),
            title_entry: adw::EntryRow::default(),
            author_entry: adw::EntryRow::default(),
            cover_entry: adw::EntryRow::default(),
            read_by_nika_switch: adw::SwitchRow::default(),
            read_by_petya_switch: adw::SwitchRow::default(),
            description_view: gtk::TextView::new(),
            cover_bytes: Vec::new(),
            cover_preview: gtk::Picture::new(),
            cover_placeholder: gtk::Frame::new(None),
            cover_placeholder_label,
            provider: SearchProvider::OpenLibrary,
            search_entry: gtk::SearchEntry::new(),
            search_button: gtk::Button::new(),
            openlibrary_btn,
            google_books_btn,
            results_list: gtk::ListBox::new(),
            results_scroll: gtk::ScrolledWindow::new(),
            results: Vec::new(),
        };

        let title_entry = &model.title_entry;
        let author_entry = &model.author_entry;
        let cover_entry = &model.cover_entry;
        let read_by_nika_switch = &model.read_by_nika_switch;
        let read_by_petya_switch = &model.read_by_petya_switch;
        let description_view = &model.description_view;
        let cover_preview = &model.cover_preview;
        let cover_placeholder = &model.cover_placeholder;
        let cover_placeholder_label = &model.cover_placeholder_label;
        let search_entry = &model.search_entry;
        let search_button = &model.search_button;
        let openlibrary_btn = &model.openlibrary_btn;
        let google_books_btn = &model.google_books_btn;
        let results_list = &model.results_list;
        let results_scroll = &model.results_scroll;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            DialogCmd::SearchResult(msg) => {
                self.search_button.set_sensitive(true);
                self.search_button.set_label("Search");

                let results = match msg {
                    Ok(results) => results,
                    Err(e) => {
                        let _ = sender.output(AddBookDialogOutput::Error(e));
                        return;
                    }
                };

                self.clear_results();
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
                    self.results_list.append(&row);
                }
                self.results_scroll.set_visible(!self.results.is_empty());
            }

            DialogCmd::CoverBytes(Ok(raw)) => {
                let bytes = gtk::glib::Bytes::from(&raw[..]);
                match gtk::gdk::Texture::from_bytes(&bytes) {
                    Ok(tex) => {
                        self.cover_preview.set_paintable(Some(&tex));
                        self.cover_placeholder.set_visible(false);
                        self.cover_preview.set_visible(true);
                        self.cover_bytes = raw;
                    }
                    Err(e) => eprintln!("cover decode failed: {e}"),
                }
            }
            DialogCmd::CoverBytes(Err(e)) => eprintln!("cover fetch failed: {e}"),
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AddBookDialogMsg::Cancel => {
                self.clear();
                self.window.set_visible(false);
            }

            AddBookDialogMsg::SetProvider(p) => {
                self.provider = p;
            }

            AddBookDialogMsg::Save => {
                let mut has_error = false;
                let draft = self.draft();

                self.title_entry.remove_css_class("error");
                self.author_entry.remove_css_class("error");

                if draft.title.is_empty() {
                    self.title_entry.add_css_class("error");
                    has_error = true;
                }
                if draft.author.is_empty() {
                    self.author_entry.add_css_class("error");
                    has_error = true;
                }
                if has_error {
                    return;
                }

                let _ = sender.output(AddBookDialogOutput::Save(draft));
                self.clear();
                self.window.set_visible(false);
            }

            AddBookDialogMsg::Search => {
                let q = self.search_entry.text().trim().to_string();
                if q.is_empty() {
                    return;
                }
                self.search_button.set_sensitive(false);
                self.search_button.set_label("Searching...");

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
                    DialogCmd::SearchResult(res)
                });
            }

            AddBookDialogMsg::SelectResult(i) => {
                let Some(d) = self.results.get(i) else {
                    return;
                };

                self.title_entry.set_text(&d.title);
                self.author_entry.set_text(&d.author);
                self.cover_entry.set_text(&d.cover_url);
                self.description_view.buffer().set_text(&d.description);

                self.results_scroll.set_visible(false);

                // Trigger cover fetch or display "No cover available"
                sender.input(AddBookDialogMsg::CoverUrlChanged(d.cover_url.clone()));
            }

            AddBookDialogMsg::CoverUrlChanged(url) => {
                let url = url.trim().to_string();
                if url.is_empty() {
                    self.show_cover_placeholder(Some("No cover image\navailable"));
                    return;
                }
                sender
                    .oneshot_command(async move { DialogCmd::CoverBytes(fetch_bytes(&url).await) });
            }

            AddBookDialogMsg::ClearSearchResults => self.clear_results(),
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

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent("nlib/0.1.0")
        .build()?;
    Ok(client.get(url).send().await?.bytes().await?.to_vec())
}
