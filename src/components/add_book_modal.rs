use dioxus::prelude::*;

use crate::models::BookDraft;
use crate::services::openlibrary::search_openlibrary;

#[component]
pub fn AddBookModal(on_close: EventHandler<()>, on_save: EventHandler<BookDraft>) -> Element {
    let mut title = use_signal(String::new);
    let mut author = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut cover_url = use_signal(String::new);

    let handle_select = move |book: BookDraft| {
        title.set(book.title);
        author.set(book.author);
        description.set(book.description.unwrap_or_default());
        cover_url.set(book.cover_url.unwrap_or_default());
    };

    let handle_save = move |_| todo!();

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
                BookFormFields { title, author, cover_url, description }

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
fn BookFormFields(
    title: Signal<String>,
    author: Signal<String>,
    cover_url: Signal<String>,
    description: Signal<String>,
) -> Element {
    rsx! {
        div { class: "form-row-with-preview",
            // Cover Preview Box
            div { class: "modal-cover-preview",
                if !cover_url.read().trim().is_empty() {
                    img { class: "modal-cover-img", src: "{cover_url}", alt: "Cover preview" }
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
                        value: "{title}",
                        oninput: move |evt| title.set(evt.value()),
                    }
                }
                label { class: "form-group",
                    "Author *"
                    input {
                        class: "form-input",
                        placeholder: "e.g. Frank Herbert",
                        value: "{author}",
                        oninput: move |evt| author.set(evt.value()),
                    }
                }
                label { class: "form-group",
                    "Cover Image URL (optional)"
                    input {
                        class: "form-input",
                        r#type: "url",
                        placeholder: "https://...",
                        value: "{cover_url}",
                        oninput: move |evt| cover_url.set(evt.value()),
                    }
                }
            }
        }

        label { class: "form-group",
            "Description"ia
            textarea {
                class: "form-textarea",
                placeholder: "Short summary...",
                value: "{description}",
                oninput: move |evt| description.set(evt.value()),
            }
        }
    }
}
