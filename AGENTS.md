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
    covers.rs                 cover cache: path derived from id, existence check, save/download
  errors.rs                   (currently empty/commented — implement AppError or delete)
assets/
  style.css                   Catppuccin Macchiato; loaded via include_str! in main
```

## Relm4 mental model (read before editing views)

`view!` runs **once**, at init, building a real widget tree. It is never re-run.
State changes flow through the Elm loop: input message → `update` mutates the
model → generated code re-applies only `#[watch]`-marked properties.

- **`#[watch]` is not optional.** An unmarked property is set once and frozen. No
  error if you forget — the UI just never updates.
- **`#[watch]` goes on the property line, not the widget line.** `gtk::Picture {
  #[watch] set_filename: … }`, never `#[watch] gtk::Picture { … }`. Misplacing it *is*
  a hard error, just a cryptic one: "Widgets can only have docs and `local`, `local_ref`,
  `wrap`, `name`, `template`, `template_child` or `root` as attribute."
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

`AddBookDialog` is `Component` for exactly this reason. `App` is too, but currently
carries `CommandOutput = ()` — its async cover work moved down into `BookCard`.

**`FactoryComponent` needs no such upgrade.** It declares its own `CommandOutput`,
`update_cmd`, `Input` and `Output`, so a grid tile can fire commands as-is. Don't
"promote" one to `Component` to get async — `FactoryVecDeque` requires
`C: FactoryComponent<Index = DynamicIndex>` and would stop compiling; you'd be
hand-managing `Vec<Controller<_>>` and FlowBox children for nothing. Two differences
from the `Component` version:

- factory `update_cmd` takes **no `&Self::Root`** (copying the 4-arg `Component`
  signature gives an arity error)
- `update_cmd_with_view` calls `update_view` **unconditionally**, so `#[watch]`
  setters re-run after a command even if the model never changed. That's what lets
  `BookCard` use `CommandOutput = ()` and still repaint after a download.

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
  selects five columns, `id` through `cover_url` (indices 0–4) — keep the SELECT and the
  `row.get(n)` calls in sync or you get `InvalidColumnIndex`.
- **Never hold the `MutexGuard` across an `.await`.** `conn()` hands back a
  `std::sync::MutexGuard`, which is `!Send`, so a future holding it fails
  `oneshot_command`'s `Future + Send` bound. The compiler catches it, but the error is a
  wall of trait bounds. Keep the split: IO in the async block, DB writes in the sync
  `update`/`update_cmd`.
- **No schema migrations.** On any schema change, delete `~/.local/share/nlib/nlib.sqlite`
  and let `open()` recreate it. Agreed tradeoff for a PoC; test rows are disposable.
- **Search filters in-memory**, not SQL, on purpose: SQLite `LIKE`/`lower()` only
  case-fold ASCII, so they miss Cyrillic (Ukrainian titles). Rust `to_lowercase()` is
  full-Unicode. Clear-and-refill the factory per keystroke — fine at ~500 books.
- **Covers: the filesystem is the source of truth.** There is no `cover_path` column.
  A cover lives at `<data dir>/covers/<id>.jpg`, so the path is *derived* from the id:
  `covers::existing_cover(id) -> Option<PathBuf>` returns `Some` only when the file is
  really there. Storing the path was worse than redundant — it was an **absolute** path,
  so a DB copied to another machine or opened as the other user pointed at nothing.
  `cover_url` stays in the DB: it's the source of record (the user may paste any URL) and
  the input for a re-download.
  - **`BookCard` fetches its own cover.** `init_model` checks `existing_cover(id)` and,
    if it's missing and `cover_url` is non-empty, spawns `download_cover`. There's nothing
    to persist afterwards, so the card needs no `Db` and `App` needs no `AppCmd`/`update_cmd`.
  - On save, if the dialog already fetched preview bytes, `save_cover_bytes` writes them
    synchronously (instant, no second request); otherwise the card downloads on first paint.
  - Self-healing falls out for free: delete a cover file and it re-downloads next launch.
    Four states (path set/null × file present/absent, two of them broken) collapsed to two.
  - `.jpg` is hardcoded in the filename regardless of what the server actually sent. Fine
    today; supporting several formats would need a stored filename or a glob.
  - Cost: one `stat` per card per `refresh_grid()`, i.e. per keystroke. Noise next to
    rebuilding the widget tree and decoding the JPEGs in the same loop.
  - Dialog invariant: `cover_bytes` is non-empty exactly when a valid preview is showing.

## Roadmap: book management (agreed 2026-09-01)

Goal: books are currently write-once — you can add and search, nothing else. Add
delete, a detail view that doubles as the editor, and read-tracking for the two users.

**Do these in order.** Each step lays plumbing the next one needs; going out of order
means building the same form or the same message path twice.

### 1. Delete a book

Smallest, and it's the foundation: it introduces factory child → parent messaging,
which steps 3 and 4 both reuse.

- `BookCard` gets a real `Output` (currently `()`), e.g. `BookCardOutput::Delete(i64)`.
- `App` swaps `.detach()` for `.forward(sender.input_sender(), …)` —
  `FactoryVecDequeConnector::forward` is at `relm4-0.11.0/src/factory/sync/collections/vec_deque.rs:461`.
  Dropping the returned handle destroys the children, so it stays on the model.
- New `Db::delete_book(id)`, then delete `covers/{id}.jpg`, then `refresh_grid()`.
- Orphaned cover files aren't a correctness bug (`AUTOINCREMENT` never reuses ids) but
  the cache would grow forever, so clean the file up.
- Expect to meet `DynamicIndex` here — it's what keeps indices valid after a removal.

### 2. Read-tracking fields

Schema and model only; display read-only on the card. Land before step 3 so the detail
form is built once against the final field set.

```sql
read_by_nika  TEXT,   -- ISO 'YYYY-MM-DD', NULL = not read
read_by_petya TEXT
```

- **Two columns, not four.** The presence of a date *is* the "has read" boolean. A
  separate flag admits `read=true, date=NULL` and `read=false, date=set` — meaningless
  states the UI would have to defend forever. Same reasoning that killed `cover_path`.
- `Option<String>` on `Book`; `.is_some()` answers "has Nika read it".
- **No normalized `reads` table.** Two known people, permanently. A join and a second
  model type buy nothing here.
- TEXT over an integer epoch because ISO strings sort correctly, so `ORDER BY
  read_by_nika` works with no date parsing.
- Today's date needs **no `chrono`/`time` dep**: `relm4::gtk::glib::DateTime::now_local()`
  then `.format("%Y-%m-%d")`.
- Touches `storage.rs` (CREATE TABLE, SELECT, `row.get(n)` indices — keep them in sync),
  `models.rs` (`Book`, maybe `BookDraft`), and the `BookCard` view for a read badge.
- Requires deleting `~/.local/share/nlib/nlib.sqlite` (see "No schema migrations").
- **Decided (2026-09-01): mark-as-read stamps the current date, full stop.** There is no
  date picker and no way to record "read it, don't remember when" — the write is a
  one-click toggle that sets today, and clearing it sets `NULL`. Deliberately minimal;
  backdating is a later extension if it turns out to be wanted, and the schema already
  allows it without a migration (any ISO date is valid in the column).

### 3. `BookView` — detail modal

A new `Component` in `components/`, modelled on `AddBookDialog` (same modal shape).
Shows every field for one book and edits them in place, including the step-2 read fields.
Opened from the grid; `App` holds the `Controller` like it already does for the dialog.

- Needs `Db::update_book(&Book)` (or a dedicated draft type).
- **Deliberately separate from `AddBookDialog`, not a shared Add|Edit mode.** The add
  flow is OpenLibrary-search-driven; this one is not. They overlap on field rows only,
  and that duplication is cheaper than a component with two personalities. Revisit only
  if the rows actually start drifting.
- **Cover invalidation:** if `cover_url` is edited, the cached `covers/{id}.jpg` is stale
  but `existing_cover` still reports it present, so nothing re-downloads. Delete the file
  on `cover_url` change and let `BookCard` re-fetch on next paint — this works precisely
  because the filesystem is the source of truth.
- Full-size cover display can't use `set_filename` for a remote URL. That's the
  bytes → `glib::Bytes` → `gdk::Texture::from_bytes` → `set_paintable` route (see the
  `gtk::Picture` trap above). `gdk::Texture` is **not `Send`** — return `Vec<u8>` from the
  command and build the texture in `update_cmd` on the main thread.

### 4. Card quick actions

An edit button (and likely a quick "mark read" toggle) directly on `BookCard`, so common
operations skip the modal. Reuses the step-1 `Output` channel — just more variants.
Deferred on purpose: the modal has to exist and be correct first.

## Open loose ends (non-functional)

- Empty-grid state: zero search results shows a blank grid; want `adw::StatusPage` in a
  `gtk::Stack`.
- Style polish against the pumpbin/Catppuccin.
- **Migrations will stop being optional.** The no-migrations rule holds only while the
  rows are disposable test data. Once there's a real shared library in there, a schema
  change needs `PRAGMA user_version` + a migration step instead of deleting the file.
