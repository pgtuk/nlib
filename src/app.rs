use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::components::add_book_dialog::{AddBookDialog, AddBookDialogOutput};
use crate::components::book_card::{BookCard, BookCardOutput};
use crate::components::book_view::{BookViewModal, BookViewOutput};
use crate::errors::AppError;
use crate::models::{Book, BookDraft, BookLocation};
use crate::services::library::LibraryService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortField {
    #[default]
    DateAdded,
    Title,
    Author,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FilterState {
    pub nika_read: bool,
    pub nika_unread: bool,
    pub petya_read: bool,
    pub petya_unread: bool,
    pub location_home: bool,
    pub location_borrowed: bool,
    pub location_dacha: bool,
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

        let location_filter_active =
            self.location_home || self.location_borrowed || self.location_dacha;
        let location_ok = if location_filter_active {
            match book.location {
                BookLocation::Home => self.location_home,
                BookLocation::Borrowed => self.location_borrowed,
                BookLocation::Dacha => self.location_dacha,
            }
        } else {
            true
        };

        nika_ok && petya_ok && location_ok
    }
}

pub struct App {
    library: LibraryService,
    displayed_books: FactoryVecDeque<BookCard>,
    query: String,
    filter: FilterState,
    sort_field: SortField,
    sort_order: SortOrder,
    add_book_modal: Controller<AddBookDialog>,
    book_view_modal: Option<Controller<BookViewModal>>,
}

impl App {
    fn refresh_grid(&mut self) -> Result<(), AppError> {
        let books = self.library.books()?;
        let mut filtered: Vec<Book> = books
            .into_iter()
            .filter(|b| {
                self.filter.matches(b)
                    && (b.title.to_lowercase().contains(&self.query)
                        || b.author.to_lowercase().contains(&self.query))
            })
            .collect();

        // Apply sorting
        filtered.sort_by(|a, b| {
            let ordering = match self.sort_field {
                SortField::DateAdded => a.id.cmp(&b.id),
                SortField::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                SortField::Author => a.author.to_lowercase().cmp(&b.author.to_lowercase()),
            };
            match self.sort_order {
                SortOrder::Asc => ordering,
                SortOrder::Desc => ordering.reverse(),
            }
        });

        let mut guard = self.displayed_books.guard();
        guard.clear();
        for book in filtered {
            guard.push_back(book);
        }
        Ok(())
    }

    fn refresh_grid_or_report(&mut self, sender: &ComponentSender<Self>) {
        if let Err(e) = self.refresh_grid() {
            sender.input(AppAction::ShowError(e));
        }
    }
}

#[derive(Debug)]
pub enum AppAction {
    Search(String),
    ToggleNikaRead(bool),
    ToggleNikaUnread(bool),
    TogglePetyaRead(bool),
    TogglePetyaUnread(bool),
    ToggleLocationHome(bool),
    ToggleLocationBorrowed(bool),
    ToggleLocationDacha(bool),
    SetSortField(SortField),
    ToggleSortOrder,
    ResetFilters,
    OpenAddDialog,
    SaveBook(BookDraft),
    OpenBook(Book),
    UpdateBook(Book),
    DeleteBook(i64),
    ExportBackup,
    ImportBackup,
    ShowInfo(String),
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
                        #[wrap(Some)]
                        set_title_widget = &gtk::Box {
                            set_visible: false,
                        },

                        pack_start = &gtk::Label {
                            set_label: "📚 Nika's lib",
                            add_css_class: "heading",
                            set_margin_end: 12,
                        },

                        pack_start = &gtk::Box {
                            set_spacing: 6,
                            set_size_request: (320, -1),

                            gtk::SearchEntry {
                                set_hexpand: true,
                                set_placeholder_text: Some("Search by title or author..."),
                                connect_search_changed[sender] => move |entry| {
                                    sender.input(AppAction::Search(entry.text().to_string()));
                                },
                            },

                            gtk::MenuButton {
                                set_icon_name: "view-filter-symbolic",
                                set_tooltip_text: Some("Filter & Sort books"),
                                #[wrap(Some)]
                                set_popover = &gtk::Popover {
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 8,
                                        set_margin_all: 10,

                                        gtk::Label {
                                            set_label: "Sort by",
                                            add_css_class: "heading",
                                            set_xalign: 0.0,
                                        },

                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Horizontal,
                                            set_spacing: 6,

                                            #[name = "sort_combo"]
                                            gtk::DropDown {
                                                set_model: Some(&gtk::StringList::new(&["Date Added", "Title", "Author"])),
                                                set_selected: 0,
                                                connect_selected_notify[sender] => move |dropdown| {
                                                    let field = match dropdown.selected() {
                                                        1 => SortField::Title,
                                                        2 => SortField::Author,
                                                        _ => SortField::DateAdded,
                                                    };
                                                    sender.input(AppAction::SetSortField(field));
                                                },
                                            },

                                            #[name = "sort_order_btn"]
                                            gtk::Button {
                                                set_icon_name: "view-sort-descending-symbolic",
                                                set_tooltip_text: Some("Toggle Ascending/Descending"),
                                                connect_clicked[sender] => move |_| {
                                                    sender.input(AppAction::ToggleSortOrder);
                                                },
                                            },
                                        },

                                        gtk::Separator {
                                            set_orientation: gtk::Orientation::Horizontal,
                                        },

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

                                        gtk::Label {
                                            set_label: "Location",
                                            add_css_class: "heading",
                                            set_xalign: 0.0,
                                        },

                                        #[name = "loc_home_check"]
                                        gtk::CheckButton {
                                            set_label: Some("Home"),
                                            connect_toggled[sender] => move |btn| {
                                                sender.input(AppAction::ToggleLocationHome(btn.is_active()));
                                            },
                                        },

                                        #[name = "loc_borrowed_check"]
                                        gtk::CheckButton {
                                            set_label: Some("Borrowed"),
                                            connect_toggled[sender] => move |btn| {
                                                sender.input(AppAction::ToggleLocationBorrowed(btn.is_active()));
                                            },
                                        },

                                        #[name = "loc_dacha_check"]
                                        gtk::CheckButton {
                                            set_label: Some("Dacha"),
                                            connect_toggled[sender] => move |btn| {
                                                sender.input(AppAction::ToggleLocationDacha(btn.is_active()));
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

                        pack_end = &gtk::MenuButton {
                            set_icon_name: "open-menu-symbolic",
                            set_tooltip_text: Some("Library options"),
                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 6,
                                    set_margin_all: 8,

                                    gtk::Button {
                                        set_label: "Export Backup (JSON)",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(AppAction::ExportBackup);
                                        },
                                    },

                                    gtk::Button {
                                        set_label: "Import Backup (JSON)",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(AppAction::ImportBackup);
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
                        set_vexpand: true,
                        set_hexpand: true,

                        #[wrap(Some)]
                        set_child = &gtk::FlowBox {
                            set_selection_mode: gtk::SelectionMode::None,
                            set_max_children_per_line: 12,
                            set_min_children_per_line: 2,
                            set_row_spacing: 12,
                            set_column_spacing: 12,
                            set_margin_all: 12,
                            set_valign: gtk::Align::Start,

                            #[local_ref]
                            books_grid -> gtk::FlowBox,
                        },
                    },
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
            });

        let displayed_books = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.input_sender(), |out| match out {
                BookCardOutput::Open(b) => AppAction::OpenBook(b),
            });

        let mut model = Self {
            library,
            displayed_books,
            query: String::new(),
            filter: FilterState::default(),
            sort_field: SortField::default(),
            sort_order: SortOrder::default(),
            add_book_modal,
            book_view_modal: None,
        };

        let books_grid = model.displayed_books.widget();
        let widgets = view_output!();

        model.refresh_grid_or_report(&sender);

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
            AppAction::OpenAddDialog => self.add_book_modal.widget().present(),
            AppAction::SaveBook(draft) => self.handle_save_book(draft, &sender),
            AppAction::Search(q) => self.handle_search(q, &sender),
            AppAction::SetSortField(field) => self.handle_set_sort_field(field, &sender),
            AppAction::ToggleSortOrder => self.handle_toggle_sort_order(widgets, &sender),
            AppAction::ToggleNikaRead(active) => self.handle_toggle_nika_read(active, &sender),
            AppAction::ToggleNikaUnread(active) => self.handle_toggle_nika_unread(active, &sender),
            AppAction::TogglePetyaRead(active) => self.handle_toggle_petya_read(active, &sender),
            AppAction::TogglePetyaUnread(active) => {
                self.handle_toggle_petya_unread(active, &sender)
            }
            AppAction::ToggleLocationHome(active) => {
                self.handle_toggle_location_home(active, &sender)
            }
            AppAction::ToggleLocationBorrowed(active) => {
                self.handle_toggle_location_borrowed(active, &sender)
            }
            AppAction::ToggleLocationDacha(active) => {
                self.handle_toggle_location_dacha(active, &sender)
            }
            AppAction::ResetFilters => self.handle_reset_filters(widgets, &sender),
            AppAction::OpenBook(book) => self.handle_open_book(book, root, &sender),
            AppAction::UpdateBook(book) => self.handle_update_book(book, &sender),
            AppAction::DeleteBook(id) => self.handle_delete_book(id, &sender),
            AppAction::ExportBackup => self.handle_export_backup(root, &sender),
            AppAction::ImportBackup => self.handle_import_backup(root, &sender),
            AppAction::ShowInfo(msg) => self.handle_show_info(&msg, widgets, &sender),
            AppAction::ShowError(err) => self.handle_show_error(&err, widgets),
        }
    }
}

impl App {
    fn handle_save_book(&mut self, draft: BookDraft, sender: &ComponentSender<Self>) {
        match self.library.add_book(&draft) {
            Ok(_) => self.refresh_grid_or_report(sender),
            Err(e) => sender.input(AppAction::ShowError(e.into())),
        }
    }

    fn handle_search(&mut self, query: String, sender: &ComponentSender<Self>) {
        self.query = query.to_lowercase();
        self.refresh_grid_or_report(sender);
    }

    fn handle_set_sort_field(&mut self, field: SortField, sender: &ComponentSender<Self>) {
        self.sort_field = field;
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_sort_order(
        &mut self,
        widgets: &mut AppWidgets,
        sender: &ComponentSender<Self>,
    ) {
        self.sort_order = match self.sort_order {
            SortOrder::Asc => {
                widgets
                    .sort_order_btn
                    .set_icon_name("view-sort-descending-symbolic");
                SortOrder::Desc
            }
            SortOrder::Desc => {
                widgets
                    .sort_order_btn
                    .set_icon_name("view-sort-ascending-symbolic");
                SortOrder::Asc
            }
        };
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_nika_read(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.filter.nika_read = active;
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_nika_unread(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.filter.nika_unread = active;
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_petya_read(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.filter.petya_read = active;
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_petya_unread(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.filter.petya_unread = active;
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_location_home(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.filter.location_home = active;
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_location_borrowed(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.filter.location_borrowed = active;
        self.refresh_grid_or_report(sender);
    }

    fn handle_toggle_location_dacha(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.filter.location_dacha = active;
        self.refresh_grid_or_report(sender);
    }

    fn handle_reset_filters(&mut self, widgets: &mut AppWidgets, sender: &ComponentSender<Self>) {
        self.filter = FilterState::default();
        self.sort_field = SortField::DateAdded;
        self.sort_order = SortOrder::Desc;
        widgets.sort_combo.set_selected(0);
        widgets
            .sort_order_btn
            .set_icon_name("view-sort-descending-symbolic");
        widgets.nika_read_check.set_active(false);
        widgets.nika_unread_check.set_active(false);
        widgets.petya_read_check.set_active(false);
        widgets.petya_unread_check.set_active(false);
        widgets.loc_home_check.set_active(false);
        widgets.loc_borrowed_check.set_active(false);
        widgets.loc_dacha_check.set_active(false);
        self.refresh_grid_or_report(sender);
    }

    fn handle_open_book(
        &mut self,
        book: Book,
        root: &adw::ApplicationWindow,
        sender: &ComponentSender<Self>,
    ) {
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

    fn handle_update_book(&mut self, book: Book, sender: &ComponentSender<Self>) {
        match self.library.update_book(&book) {
            Ok(()) => {
                self.book_view_modal = None;
                self.refresh_grid_or_report(sender);
            }
            Err(e) => sender.input(AppAction::ShowError(e.into())),
        }
    }

    fn handle_delete_book(&mut self, id: i64, sender: &ComponentSender<Self>) {
        match self.library.delete_book(id) {
            Ok(()) => {
                self.book_view_modal = None;
                self.refresh_grid_or_report(sender);
            }
            Err(e) => sender.input(AppAction::ShowError(e.into())),
        }
    }

    fn handle_export_backup(&self, root: &adw::ApplicationWindow, sender: &ComponentSender<Self>) {
        let dialog = gtk::FileDialog::builder()
            .title("Export Library Backup")
            .initial_name("nlib-backup.json")
            .build();

        let sender_clone = sender.clone();
        let library = self.library.clone();
        dialog.save(Some(root), gtk::gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    match library.export_to_json(&path) {
                        Ok(count) => sender_clone.input(AppAction::ShowInfo(format!(
                            "Successfully exported {count} books to backup."
                        ))),
                        Err(e) => sender_clone.input(AppAction::ShowError(e.into())),
                    }
                }
            }
        });
    }

    fn handle_import_backup(&self, root: &adw::ApplicationWindow, sender: &ComponentSender<Self>) {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("JSON files"));
        filter.add_pattern("*.json");

        let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title("Import Library Backup")
            .filters(&filters)
            .build();

        let sender_clone = sender.clone();
        let library = self.library.clone();
        dialog.open(Some(root), gtk::gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    match library.import_from_json(&path) {
                        Ok(summary) => {
                            let mut msg = format!("Imported {} books.", summary.imported);
                            if summary.skipped_duplicates > 0 {
                                msg.push_str(&format!(
                                    " (Skipped {} existing duplicates)",
                                    summary.skipped_duplicates
                                ));
                            }
                            sender_clone.input(AppAction::ShowInfo(msg));
                        }
                        Err(e) => sender_clone.input(AppAction::ShowError(e.into())),
                    }
                }
            }
        });
    }

    fn handle_show_info(
        &mut self,
        msg: &str,
        widgets: &mut AppWidgets,
        sender: &ComponentSender<Self>,
    ) {
        self.refresh_grid_or_report(sender);
        let toast = adw::Toast::new(msg);
        toast.set_timeout(4);
        widgets.toast_overlay.add_toast(toast);
    }

    fn handle_show_error(&self, err: &AppError, widgets: &mut AppWidgets) {
        let toast = adw::Toast::new(&err.to_string());
        toast.set_timeout(5);
        widgets.toast_overlay.add_toast(toast);
    }
}
