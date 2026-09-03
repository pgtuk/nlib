# Nika's Lib (nlib) 📚

A fast, lightweight desktop personal library manager built with **Rust**, **GTK4**, **Libadwaita**, and **Relm4**.

---

## Features

- 📖 **Catalog & Manage Books**: Add, view, edit details, and track reading statuses independently for Nika and Petya.
- 🔍 **Multi-Source Book Search**:
  - **OpenLibrary**: Free community database (searches titles, authors, and ISBNs out-of-the-box).
  - **Google Books**: Rich international metadata with automatic high-resolution cover upscaling (supports title, author, and `isbn:` searches).
- 🖼️ **High-Resolution Covers**: Local caching with dedicated preview modal cards.
- 🏷️ **Filters & Sorting**:
  - Filter by reading statuses (Read / Unread).
  - Sort by **Date Added**, **Title**, or **Author** (Ascending / Descending).
  - Instant client-side search across titles and authors.
- 💾 **Backup & Restore**:
  - Export your entire library to JSON backup.
  - Import JSON backups with duplicate detection.
- 🔄 **In-App Auto-Updates**: Checks GitHub releases for updates and applies seamless in-place executable upgrades.
- 🎨 **Modern Libadwaita UI**: Responsive grid layout styled with Catppuccin Macchiato colors.

---

## Configuration

For local development, you can optionally configure keys in a `.env` file in the project root:

```ini
# Optional: Override Google Books API key for higher rate limits during local development
GOOGLE_BOOKS_API_KEY="your_google_books_api_key_here"
```

> **Note**: For release builds published via GitHub Actions, `GH_TOKEN` (for private repo release access) and `GOOGLE_BOOKS_API_KEY` are baked directly into the binary from GitHub Secrets at compile-time. End users and installed builds work out-of-the-box with zero configuration or tokens needed.

---

## In-App Auto-Updates

`nlib` includes built-in update checking and self-upgrading:
- **Check for updates**: On launch (or via the About/Menu action), the app queries the GitHub Releases API for newer semver tags.
- **Atomic updates**: When an update is accepted, the new binary is downloaded to a temporary file, granted executable permissions (`0755`), and atomically replaces the currently running executable via `rename(2)`. The updated version runs on next launch.

---

## Releases & Publishing

The repository uses [`cargo-release`](https://github.com/crate-ci/cargo-release) along with GitHub Actions to automate versioning and binary releases.

### Prerequisites
Install `cargo-release` locally:
```bash
cargo install cargo-release
```

### Making a Release
Ensure you are on the `main` branch with a clean working tree, then run:

```bash
# Patch release (e.g. 1.0.0 -> 1.0.1)
cargo release patch --execute

# Minor release (e.g. 1.0.1 -> 1.1.0)
cargo release minor --execute

# Major release (e.g. 1.1.0 -> 2.0.0)
cargo release major --execute
```

> **Dry run**: Omit `--execute` to preview the version bump, commit, and tags without making changes.

When pushed, the GitHub Actions release workflow builds the optimized release binary with embedded secrets and attaches it to the GitHub Release.

---

## Development & Build

### Requirements
- Rust 1.80+ (or latest stable)
- GTK4 (`libgtk-4-dev`)
- Libadwaita (`libadwaita-1-dev`)

### Run in Debug Mode
```bash
cargo run
```

### Build Release Binary
```bash
cargo build --release
```
The compiled binary will be placed at `target/release/nlib`.

### Release Profile Optimizations
The release build profile in `Cargo.toml` is tuned for minimal binary size and maximum performance:
- `opt-level = "z"`: Optimizes aggressively for minimal binary footprint.
- `lto = true`: Full Link-Time Optimization across all crates.
- `codegen-units = 1`: Single code generation unit allowing deeper inter-procedural compiler optimizations.
- `strip = true`: Strips debug symbols and symbol tables from the output binary.
- `panic = "abort"`: Removes stack unwinding landing pads and runtime overhead.
