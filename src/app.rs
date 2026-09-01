use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::components::add_book_dialog::{AddBookDialog, AddBookDialogOutput};
use crate::components::book_card::{BookCard, BookCardOutput};
use crate::components::book_view::{BookViewModal, BookViewOutput};
use crate::errors::AppError;
use crate::models::{Book, BookDraft};
use crate::services::library::LibraryService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FilterState {
    pub nika_read: bool,
    pub nika_unread: bool,
    pub petya_read: bool,
    pub petya_unread: bool,
}

impl FilterState {
    pub fn matches(&self, book: &Book) -> bool {
        let nika_ok = match (self.nika_read, self.nika_unread) {
            (true, false) => book.read_by_nika,
            (false, true) => !book.read_by_nika,
            _ => true,
        };

        let petya_ok = match (self.petya_read, self.petya_unread) {
            (true, false) => book.read_by_petya,
            (false, true) => !book.read_by_petya,
            _ => true,
        };

        nika_ok && petya_ok
    }
}

pub struct App {
    library: LibraryService,
    displayed_books: FactoryVecDeque<BookCard>,
    query: String,
    filter: FilterState,
    add_book_modal: Controller<AddBookDialog>,
    book_view_modal: Option<Controller<BookViewModal>>,
}

impl App {
    fn refresh_grid(&mut self) -> Result<(), AppError> {
        let books = self.library.books()?;
        let mut guard = self.displayed_books.guard();
        guard.clear();
        for book in books.into_iter().filter(|b| {
            self.filter.matches(b)
                && (b.title.to_lowercase().contains(&self.query)
                    || b.author.to_lowercase().contains(&self.query))
        }) {
            guard.push_back(book);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AppAction {
    Search(String),
    ToggleNikaRead(bool),
    ToggleNikaUnread(bool),
    TogglePetyaRead(bool),
    TogglePetyaUnread(bool),
    ResetFilters,
    OpenAddDialog,
    SaveBook(BookDraft),
    OpenBook(Book),
    UpdateBook(Book),
    DeleteBook(i64),
    ShowError(AppError),
}

#[relm4::component(pub)]
impl Component for App {
    type Init = LibraryService;
    type Input = AppAction;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("Nika's lib"),
            set_icon_name: Some("dev.nlib.app"),
            set_default_size: (960, 720),

            #[name = "toast_overlay"]
            adw::ToastOverlay {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::HeaderBar {
                        pack_start = &gtk::Label {
                            set_label: "📚 Nika's lib",
                            add_css_class: "heading"
                        },

                        #[wrap(Some)]
                        set_title_widget = &adw::Clamp {
                            set_maximum_size: 400,

                            gtk::Box {
                                set_spacing: 6,

                                gtk::SearchEntry {
                                    set_hexpand: true,
                                    set_placeholder_text: Some("Search by title or author..."),
                                    connect_search_changed[sender] => move |entry| {
                                        sender.input(AppAction::Search(entry.text().to_string()));
                                    },
                                },

                                gtk::MenuButton {
                                    set_icon_name: "view-filter-symbolic",
                                    set_tooltip_text: Some("Filter books"),
                                    #[wrap(Some)]
                                    set_popover = &gtk::Popover {
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_spacing: 8,
                                            set_margin_all: 10,

                                            gtk::Label {
                                                set_label: "Nika",
                                                add_css_class: "heading",
                                                set_xalign: 0.0,
                                            },

                                            #[name = "nika_read_check"]
                                            gtk::CheckButton {
                                                set_label: Some("Read"),
                                                connect_toggled[sender] => move |btn| {
                                                    sender.input(AppAction::ToggleNikaRead(btn.is_active()));
                                                },
                                            },

                                            #[name = "nika_unread_check"]
                                            gtk::CheckButton {
                                                set_label: Some("Unread"),
                                                connect_toggled[sender] => move |btn| {
                                                    sender.input(AppAction::ToggleNikaUnread(btn.is_active()));
                                                },
                                            },

                                            gtk::Separator {
                                                set_orientation: gtk::Orientation::Horizontal,
                                            },

                                            gtk::Label {
                                                set_label: "Petya",
                                                add_css_class: "heading",
                                                set_xalign: 0.0,
                                            },

                                            #[name = "petya_read_check"]
                                            gtk::CheckButton {
                                                set_label: Some("Read"),
                                                connect_toggled[sender] => move |btn| {
                                                    sender.input(AppAction::TogglePetyaRead(btn.is_active()));
                                                },
                                            },

                                            #[name = "petya_unread_check"]
                                            gtk::CheckButton {
                                                set_label: Some("Unread"),
                                                connect_toggled[sender] => move |btn| {
                                                    sender.input(AppAction::TogglePetyaUnread(btn.is_active()));
                                                },
                                            },

                                            gtk::Separator {
                                                set_orientation: gtk::Orientation::Horizontal,
                                            },

                                            gtk::Button {
                                                set_label: "Reset",
                                                connect_clicked[sender] => move |_| {
                                                    sender.input(AppAction::ResetFilters);
                                                },
                                            },
                                        },
                                    },
                                },
                            },
                        },

                        pack_end = &gtk::Button {
                            set_label: "+ Add Book",
                            add_css_class: "suggested-action",
                            connect_clicked => AppAction::OpenAddDialog,
                        },
                    },

                    gtk::ScrolledWindow {
                        set_vexpand: true,          // without this it collapses to zero height

                        #[local_ref]
                        book_flow_box -> gtk::FlowBox {
                            set_valign: gtk::Align::Start,
                            set_homogeneous: true,
                            set_selection_mode: gtk::SelectionMode::None,
                            set_row_spacing: 12,
                            set_column_spacing: 12,
                            set_margin_start: 16,
                            set_margin_end: 16,
                            set_margin_top: 16,
                            set_margin_bottom: 16,
                        }
                    }
                }
            }
        }
    }

    fn init(
        library: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let add_book_modal = AddBookDialog::builder()
            .transient_for(&root)
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                AddBookDialogOutput::Save(draft) => AppAction::SaveBook(draft),
                AddBookDialogOutput::Error(err) => AppAction::ShowError(AppError::Search(err)),
            });

        let mut model = App {
            library,
            displayed_books: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |out| match out {
                    BookCardOutput::Open(book) => AppAction::OpenBook(book),
                },
            ),
            query: String::new(),
            filter: FilterState::default(),
            add_book_modal,
            book_view_modal: None,
        };

        if let Err(e) = model.refresh_grid() {
            sender.input(AppAction::ShowError(e));
        }

        let book_flow_box = model.displayed_books.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            AppAction::OpenAddDialog => {
                self.add_book_modal.widget().present();
            }
            AppAction::SaveBook(draft) => match self.library.add_book(&draft) {
                Ok(_) => {
                    if let Err(e) = self.refresh_grid() {
                        sender.input(AppAction::ShowError(e));
                    }
                }
                Err(e) => {
                    sender.input(AppAction::ShowError(e.into()));
                }
            },
            AppAction::Search(q) => {
                self.query = q.to_lowercase();
                if let Err(e) = self.refresh_grid() {
                    sender.input(AppAction::ShowError(e));
                }
            }
            AppAction::ToggleNikaRead(active) => {
                self.filter.nika_read = active;
                if let Err(e) = self.refresh_grid() {
                    sender.input(AppAction::ShowError(e));
                }
            }
            AppAction::ToggleNikaUnread(active) => {
                self.filter.nika_unread = active;
                if let Err(e) = self.refresh_grid() {
                    sender.input(AppAction::ShowError(e));
                }
            }
            AppAction::TogglePetyaRead(active) => {
                self.filter.petya_read = active;
                if let Err(e) = self.refresh_grid() {
                    sender.input(AppAction::ShowError(e));
                }
            }
            AppAction::TogglePetyaUnread(active) => {
                self.filter.petya_unread = active;
                if let Err(e) = self.refresh_grid() {
                    sender.input(AppAction::ShowError(e));
                }
            }
            AppAction::ResetFilters => {
                self.filter = FilterState::default();
                widgets.nika_read_check.set_active(false);
                widgets.nika_unread_check.set_active(false);
                widgets.petya_read_check.set_active(false);
                widgets.petya_unread_check.set_active(false);
                if let Err(e) = self.refresh_grid() {
                    sender.input(AppAction::ShowError(e));
                }
            }
            AppAction::OpenBook(book) => {
                let modal = BookViewModal::builder()
                    .transient_for(root)
                    .launch(book)
                    .forward(sender.input_sender(), |out| match out {
                        BookViewOutput::Save(book) => AppAction::UpdateBook(book),
                        BookViewOutput::Delete(id) => AppAction::DeleteBook(id),
                    });
                modal.widget().present();
                self.book_view_modal = Some(modal);
            }
            AppAction::UpdateBook(book) => match self.library.update_book(&book) {
                Ok(()) => {
                    self.book_view_modal = None;
                    if let Err(e) = self.refresh_grid() {
                        sender.input(AppAction::ShowError(e));
                    }
                }
                Err(e) => {
                    sender.input(AppAction::ShowError(e.into()));
                }
            },
            AppAction::DeleteBook(id) => match self.library.delete_book(id) {
                Ok(()) => {
                    self.book_view_modal = None;
                    if let Err(e) = self.refresh_grid() {
                        sender.input(AppAction::ShowError(e));
                    }
                }
                Err(e) => {
                    sender.input(AppAction::ShowError(e.into()));
                }
            },
            AppAction::ShowError(err) => {
                let toast = adw::Toast::new(&err.to_string());
                toast.set_timeout(5);
                widgets.toast_overlay.add_toast(toast);
            }
        }
    }
}
