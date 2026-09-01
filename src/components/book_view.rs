use crate::models::Book;
use relm4::adw::prelude::*;
use relm4::{adw, gtk};

use relm4::prelude::*;

pub struct BookViewModal {
    book: Book,
}

#[derive(Debug)]
pub enum BookViewInput {
    DeleteRequested,
    DeleteConfirmed,
}

#[derive(Debug)]
pub enum BookViewOutput {
    Delete(i64),
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
            set_default_size: (480, 600),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                adw::HeaderBar {},
            }
        }
    }

    fn init(
        book: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = BookViewModal { book };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
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
            }
        }
    }
}
