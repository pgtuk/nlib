use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::components::add_book_dialog::{AddBookDialog, AddBookDialogOutput};
use crate::components::book_card::{BookCard, BookCardOutput};
use crate::components::book_view::{BookViewModal, BookViewOutput};
use crate::errors::AppError;
use crate::models::{Book, BookDraft};
use crate::services::library::LibraryService;

pub struct App {
    library: LibraryService,
    displayed_books: FactoryVecDeque<BookCard>,
    query: String,
    add_book_modal: Controller<AddBookDialog>,
    book_view_modal: Option<Controller<BookViewModal>>,
}

impl App {
    fn refresh_grid(&mut self) -> Result<(), AppError> {
        let books = self.library.books()?;
        let mut guard = self.displayed_books.guard();
        guard.clear();
        for book in books.into_iter().filter(|b| {
            b.title.to_lowercase().contains(&self.query)
                || b.author.to_lowercase().contains(&self.query)
        }) {
            guard.push_back(book);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AppAction {
    Search(String),
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
                        set_title_widget = &gtk::SearchEntry {
                            set_placeholder_text: Some("Search by title or author..."),
                            connect_search_changed[sender] => move |entry| {
                                sender.input(AppAction::Search(entry.text().to_string()));
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
                            // ...
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
