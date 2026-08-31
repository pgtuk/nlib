use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};
use std::path::PathBuf;

use crate::components::add_book_dialog::{AddBookDialog, AddBookDialogOutput};
use crate::components::book_card::BookCard;
use crate::models::{Book, BookDraft};
use crate::services::covers;
use crate::services::storage::Db;

pub struct App {
    all_books: Vec<Book>,
    books: FactoryVecDeque<BookCard>,
    query: String,
    db: Db,
    dialog: Controller<AddBookDialog>,
}

impl App {
    fn refresh_grid(&mut self) {
        let mut guard = self.books.guard();
        guard.clear();
        for book in self.all_books.iter().filter(|b| {
            b.title.to_lowercase().contains(&self.query)
                || b.author.to_lowercase().contains(&self.query)
        }) {
            guard.push_back(book.clone());
        }
    }
}

#[derive(Debug)]
pub enum AppMsg {
    Search(String),
    OpenAddDialog,
    SaveBook(BookDraft),
}

#[derive(Debug)]
pub enum AppCmd {
    CoverReady(i64, PathBuf),
}

#[relm4::component(pub)]
impl Component for App {
    type Init = Db;
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = AppCmd;

    view! {
        adw::ApplicationWindow {
            set_title: Some("Nika's lib"),
            set_default_size: (960, 720),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                        set_title_widget = &gtk::SearchEntry {
                            set_placeholder_text: Some("Search by title or author..."),
                            connect_search_changed[sender] => move |entry| {
                                sender.input(AppMsg::Search(entry.text().to_string()));
                            },
                        },

                        pack_end = &gtk::Button {
                            set_label: "+ Add Book",
                            add_css_class: "suggested-action",
                            connect_clicked => AppMsg::OpenAddDialog,
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

    fn init(
        db: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let all_books = db.books().expect("load books"); // .unwrap_or_default();
        let dialog = AddBookDialog::builder()
            .transient_for(&root)
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                AddBookDialogOutput::Save(draft) => AppMsg::SaveBook(draft),
            });
        let mut model = App {
            db,
            all_books,
            books: FactoryVecDeque::builder().launch_default().detach(),
            query: String::new(),
            dialog,
        };
        model.refresh_grid();
        let book_flow_box = model.books.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AppMsg::OpenAddDialog => {
                self.dialog.widget().present();
            }
            AppMsg::SaveBook(draft) => {
                let id = match self.db.add_book(&draft) {
                    Ok(id) => id,
                    Err(e) => {
                        eprintln!("failed to save book: {e}");
                        return;
                    }
                };
                self.all_books = self.db.books().unwrap_or_default();
                self.refresh_grid();

                match draft.cover_bytes {
                    Some(raw) => match covers::save_cover_bytes(id, &raw) {
                        Ok(path) => {
                            let _ = self.db.set_cover_path(id, &path.to_string_lossy());
                            self.all_books = self.db.books().unwrap_or_default();
                            self.refresh_grid();
                        }
                        Err(e) => eprintln!("save cover failed: {e}"),
                    },
                    None if !draft.cover_url.is_empty() => {
                        let url = draft.cover_url.clone();
                        sender.oneshot_command(async move {
                            match covers::download_cover(&url, id).await {
                                Ok(path) => AppCmd::CoverReady(id, path),
                                Err(e) => {
                                    eprintln!("cover download failed: {e}");
                                    AppCmd::CoverReady(id, PathBuf::new())
                                }
                            }
                        });
                    }
                    None => {}
                }
            }
            AppMsg::Search(q) => {
                self.query = q.to_lowercase();
                self.refresh_grid();
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let AppCmd::CoverReady(id, path) = msg;
        if path.as_os_str().is_empty() {
            return; // download failed — the PathBuf::new() sentinel
        }
        if let Err(e) = self.db.set_cover_path(id, &path.to_string_lossy()) {
            eprintln!("failed to store cover path: {e}");
            return;
        }
        self.all_books = self.db.books().unwrap_or_default();
        self.refresh_grid();
    }
}
