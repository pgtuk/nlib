use dioxus::prelude::*;

use crate::models::BookDraft;
use crate::services::openlibrary::search_openlibrary;

#[component]
pub fn AddBookModal(on_close: EventHandler<()>, on_save: EventHandler<BookDraft>) -> Element {
    let mut draft = use_signal(BookDraft::default);

    let handle_select = move |book: BookDraft| {
        draft.set(book);
    };

    let handle_save = move |_| on_save.call(draft());

    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-card",
                // 1. Header
                div { class: "modal-header",
                    h2 { "Add New Book" }
                    button { class: "btn-close", onclick: move |_| on_close.call(()), "✕" }
                }

                // 2. Online Search
                OnlineSearchSection { on_select: handle_select }

                div { class: "divider", span { "Or fill manually" } }

                // 3. Form fields + Cover preview
                BookFormFields { draft }

                // 4. Action buttons
                div { class: "modal-actions",
                    button { class: "btn-secondary", onclick: move |_| on_close.call(()), "Cancel" }
                    button { class: "btn-primary", onclick: handle_save, "Save Book" }
                }
            }
        }
    }
}

#[component]
fn OnlineSearchSection(on_select: EventHandler<BookDraft>) -> Element {
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<BookDraft>::new);
    let mut is_loading = use_signal(|| false);

    let mut do_search = move || {
        let q = query.read().trim().to_string();
        if q.is_empty() {
            return;
        }
        is_loading.set(true);
        spawn(async move {
            if let Ok(res) = search_openlibrary(&q).await {
                results.set(res);
            }
            is_loading.set(false);
        });
    };

    rsx! {
        div { class: "online-search-section",
            div { class: "search-input-group",
                input {
                    class: "form-input search-bar-input",
                    r#type: "text",
                    placeholder: "Search OpenLibrary by title or author...",
                    value: "{query}",
                    oninput: move |evt| query.set(evt.value()),
                    onkeydown: move |evt| if evt.key() == Key::Enter { do_search(); },
                }
                button {
                    class: "btn-primary",
                    disabled: is_loading(),
                    onclick: move |_| do_search(),
                    if is_loading() { "Searching..." } else { "Search" }
                }
            }

            if !results.read().is_empty() {
                div { class: "search-results-list",
                    for book in results.read().iter() {
                        div {
                            class: "search-result-item",
                            onclick: {
                                let b = book.clone();
                                move |_| {
                                    on_select.call(b.clone());
                                    results.write().clear();
                                }
                            },
                            div { class: "result-title", "{book.title}" }
                            div { class: "result-author", "by {book.author}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BookFormFields(draft: Signal<BookDraft>) -> Element {
    rsx! {
        div { class: "form-row-with-preview",
            // Cover Preview Box
            div { class: "modal-cover-preview",
                if !draft.read().cover_url.trim().is_empty() {
                    img { class: "modal-cover-img", src: "{draft.read().cover_url}", alt: "Cover preview" }
                } else {
                    div { class: "modal-cover-placeholder",
                        span { "📖" }
                        span { "No Cover" }
                    }
                }
            }

            // Input Fields
            div { class: "form-fields-column",
                label { class: "form-group",
                    "Title *"
                    input {
                        class: "form-input",
                        placeholder: "e.g. Dune",
                        value: "{draft.read().title}",
                        oninput: move |evt| draft.write().title = evt.value(),
                    }
                }
                label { class: "form-group",
                    "Author *"
                    input {
                        class: "form-input",
                        placeholder: "e.g. Frank Herbert",
                        value: "{draft.read().author}",
                        oninput: move |evt| draft.write().author = evt.value(),
                    }
                }
                label { class: "form-group",
                    "Cover Image URL (optional)"
                    input {
                        class: "form-input",
                        r#type: "url",
                        placeholder: "https://...",
                        value: "{draft.read().cover_url}",
                        oninput: move |evt| draft.write().cover_url = evt.value(),
                    }
                }
            }
        }

        label { class: "form-group",
            "Description"
            textarea {
                class: "form-textarea",
                placeholder: "Short summary...",
                value: "{draft.read().description}",
                oninput: move |evt| draft.write().description = evt.value(),
            }
        }
    }
}
