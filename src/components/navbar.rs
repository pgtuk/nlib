use dioxus::prelude::*;

#[component]
pub fn NavBar(search_query: Signal<String>, on_add_click: EventHandler<()>) -> Element {
    rsx! {
        header { class: "navbar",
            h1 { class: "brand-title", "📚 Nika's lib" }
            input {
                class: "search-input",
                r#type: "text",
                placeholder: "Search by title or author...",
                value: "{search_query}",
                oninput: move |evt| search_query.set(evt.value()),
            }
            button {
                class: "btn-primary",
                onclick: move |_| on_add_click.call(()),
                "+ Add Book"
            }
        }
    }
}
