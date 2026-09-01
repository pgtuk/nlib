# nlib — agent guide

Personal book library (desktop) for two users. GTK4 + libadwaita via **Relm4**.
Migrated off Dioxus; ignore any Dioxus/RSX/`use_signal` habits — none of it applies.

Petro writes the code and is learning Rust. Guide and explain; don't implement
whole features for him unless he explicitly asks.

## Stack

| | version | notes |
|---|---|---|
| relm4 | 0.11 | features `["libadwaita", "gnome_50"]` |
| gtk4-rs | 0.11.4 | reach it via `relm4::gtk`, never a direct dep |
| libadwaita-rs | 0.9.2 | via `relm4::adw` |
| system GTK / libadwaita | 4.22 / 1.9.3 | `gnome_50` floors at GTK 4.20 / adw 1.9 |
| rustc | 1.98 | relm4 0.11 needs ≥1.93 |

`gnome_50` means the binary won't start on older GNOME. Both target machines are
current; if that changes, drop to `gnome_46` (adw 1.5).

Never add `gtk4`/`libadwaita` as direct deps — use `relm4::{gtk, adw}` (the
prelude re-exports both). Mismatched gtk crate versions give unreadable type errors.

## Run / debug

- `cargo run` — desktop app.
- `GTK_DEBUG=interactive cargo run` — live widget inspector (GTK's devtools). First
  thing to reach for when a widget is the wrong size, missing, or unstyled: walk the
  tree, check the CSS tab. Learn it before hand-debugging layout.
- Working examples on disk (match our exact version, beat the website):
  `~/.cargo/registry/src/*/relm4-0.11.0/examples/` — `factory.rs`, `entry.rs` are closest to our patterns.
- When unsure of an API, `grep` the crate source under `~/.cargo/registry/src/*/{relm4,gtk4,gdk4,libadwaita}-*/` rather than guessing. Signatures verified this way beat memory.

## Layout

```
src/
  main.rs                     startup only: Db::open, RelmApp, set_global_css
  app.rs                      App — root Component (window, headerbar, grid, save flow)
  components/
    book_card.rs              BookCard — FactoryComponent, one grid tile
    add_book_dialog.rs        AddBookDialog — Component, the add dialog (search + preview)
  models.rs                   Book (from DB), BookDraft (form input + optional cover bytes)
  services/
    storage.rs                Db: Arc<Mutex<Connection>>, Clone + Send
    openlibrary.rs            async OpenLibrary search
    covers.rs                 download / save cover thumbnails
  errors.rs                   (currently empty/commented — implement AppError or delete)
assets/
  style.css                   Catppuccin Macchiato; loaded via include_str! in main
```

## Relm4 mental model (read before editing views)

`view!` runs **once**, at init, building a real widget tree. It is never re-run.
State changes flow through the Elm loop: input message → `update` mutates the
model → generated code re-applies only `#[watch]`-marked properties.

- **`#[watch]` is not optional.** An unmarked property is set once and frozen. No
  error if you forget — the UI just never updates. (Our cards use `Input = ()` so
  nothing changes post-build; no `#[watch]` needed there.)
- **No conditional rendering in the view.** No `if cond { Widget }`. Widgets live
  from birth to death; toggle `set_visible`, use a `gtk::Stack`, or launch a
  separate component. (See the cover placeholder/preview visibility toggle in the dialog.)
- **State is one struct** (the "model"), not scattered signals. Every field a view
  reads, plus every widget handle `update` needs to touch, lives on it.

"Model" always means the struct implementing the component trait in the file
you're editing — `App`, `AddBookDialog`, or `BookCard`. There are three.

### SimpleComponent vs Component

`SimpleComponent` pins `CommandOutput = ()` — no async results. The moment you need
`sender.oneshot_command(...)` (any network/IO), upgrade to `Component`:

- `impl Component for X` (not `SimpleComponent`)
- add `type CommandOutput = ...;`
- `update` gains a 4th param: `_root: &Self::Root`
- add `fn update_cmd(&mut self, msg, sender, root)` — **provided method with an empty
  default**, so forgetting it silently drops your command result (no error). The async
  future resolves to `CommandOutput`, delivered to `update_cmd` back on the main thread.

Both `App` and `AddBookDialog` are `Component` for exactly this reason.

### Component checklist (the macro fills the rest)

Macros annotate the **`impl`**, never the struct: `#[relm4::component(pub)]`,
`#[relm4::factory(pub)]`. Use `(pub)` for cross-module components — it also makes the
generated `XWidgets` struct public, which a private impl would choke on.

A `view!` block is **required** inside the impl — it's the macro's input. Without it
you get "missing `Root` / `init_root` / `init_widgets`" errors, because those are
generated *from* the view. Factory views need an explicit `root = gtk::Box { .. }`.

Associated types you write: `Init`, `Input`, `Output`, `CommandOutput`, and (factory)
`ParentWidget`. `Root`, `Widgets`, `Index` are macro-generated — don't declare them.

### `#[local_ref]` — three parts, all same name

To have `update` touch a widget, the model owns it and the view adopts it. All four
must use the identical identifier:

1. model field: `foo: gtk::Entry`
2. `init` construction: `foo: gtk::Entry::new()`
3. `init` binding: `let foo = &model.foo;`
4. view: `#[local_ref] foo -> gtk::Entry { .. }`

Miss #3 and you get "cannot find value `foo`" (rustc even notes a field by that name
exists). Nested `#[local_ref]` works (e.g. a `ListBox` inside a `ScrolledWindow`).

Child → parent: `.forward(sender.input_sender(), |out| ParentMsg::from(out))`. Store
the returned `Controller` on the parent model — dropping it destroys the child.

## GTK / adw traps (all bit us at least once)

- **Single-child containers silently drop extra children.** `adw::Window`,
  `gtk::Window`, `gtk::ScrolledWindow`, `gtk::Frame`, `gtk::ListBoxRow`, `gtk::Button`
  hold ONE child (relm4 maps to `set_content`/`set_child`). Add two and the last wins,
  no warning. Multi-child: `gtk::Box`, `Grid`, `FlowBox`, `ListBox`, `adw::PreferencesGroup`.
- **Nothing scrolls implicitly.** Content taller than the window is clipped, not
  scrolled. Wrap in `gtk::ScrolledWindow` + `set_vexpand: true` on it.
- **Labels don't wrap or ellipsize by default,** and a long one blows out layout. Need
  `set_wrap` + `set_max_width_chars` (or a size request) + `set_ellipsize` + `set_lines`
  together. GTK CSS has no `text-align` — use `set_xalign: 0.0`.
- **`gtk::Picture` can't load a URL** — only local files (`set_filename`, takes
  `Option<impl AsRef<Path>>`). For remote images: fetch bytes async →
  `gdk::Texture::from_bytes(&glib::Bytes)` → `Picture::set_paintable`. Give it a size
  request or it collapses.
- **`adw::ActionRow`/`EntryRow` titles default to `use_markup = true`** — a raw `&` in
  data (e.g. "Mason & Dixon") fails to render. Call `.use_markup(false)` on rows fed
  API/user data. `gtk::Label` defaults markup **off** (opposite).
- **`set_global_css` must run after GTK init** (after `RelmApp::new`, before `.run`).
  Called earlier it panics: `Display::default().unwrap()` on `None`.
- **GTK CSS ≠ web CSS.** No flexbox/grid/`gap`/`text-align`. Selectors target widget
  node names (`box`, `label`, `button`, `entry`, `flowboxchild`, `window`), not tags.
  Class selectors work after `add_css_class`. Spacing is `set_spacing`/`set_margin_*`
  in Rust. libadwaita CSS custom properties (`--card-bg-color`, `--accent-bg-color`, …,
  needs adw ≥1.6) recolor everything at once — that's how `style.css` themes.
- Dark-mode warning about `gtk-application-prefer-dark-theme` comes from the user's KDE
  `~/.config/gtk-4.0/settings.ini`, not our code. Harmless; adw reads `color-scheme`.

## Domain specifics

- **`Db`** is `Arc<Mutex<Connection>>` + `Clone`, shared into components. `books()`
  selects all columns incl. `cover_path` (index 5) — keep SELECT and `row.get(n)` in
  sync or you get `InvalidColumnIndex`.
- **No schema migrations.** On any schema change, delete `~/.local/share/nlib/nlib.sqlite`
  and let `open()` recreate it. Agreed tradeoff for a PoC; test rows are disposable.
- **Search filters in-memory**, not SQL, on purpose: SQLite `LIKE`/`lower()` only
  case-fold ASCII, so they miss Cyrillic (Ukrainian titles). Rust `to_lowercase()` is
  full-Unicode. Clear-and-refill the factory per keystroke — fine at ~500 books.
- **Covers:** grid tiles show a locally cached thumbnail (`cover_path`, a file named
  `<id>.jpg` under the data dir's `covers/`); the dialog previews the remote `cover_url`
  before save. `cover_url` is the source of record (user may paste any URL); `cover_path`
  is the derived cache. On save: if the dialog already fetched bytes for the preview,
  write them synchronously (`save_cover_bytes`, instant); otherwise download async
  (`download_cover` → `AppCmd::CoverReady` → `update_cmd`). Invariant in the dialog:
  `cover_bytes` is non-empty exactly when a valid preview is showing.

## Open loose ends (non-functional)

- Empty-grid state: zero search results shows a blank grid; want `adw::StatusPage` in a
  `gtk::Stack`.
- `errors.rs` is commented out — implement `AppError` or delete.
- Style polish against the pumpbin/Catppuccin reference (`assets/example_style.png`).
- Book detail page (planned): async full-size cover fetch from `cover_url` lives here.
