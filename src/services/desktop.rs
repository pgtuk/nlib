use std::fs;

const ICON_SVG: &str = include_str!("../../assets/dev.nlib.app.svg");
const APP_ID: &str = "dev.nlib.app";

pub fn setup_desktop_integration() {
    let data_dir = relm4::gtk::glib::user_data_dir();

    // 1. Install app icon to standard icon locations
    // a. hicolor theme scalable apps
    let hicolor_apps = data_dir.join("icons/hicolor/scalable/apps");
    if let Ok(()) = fs::create_dir_all(&hicolor_apps) {
        let icon_path = hicolor_apps.join(format!("{APP_ID}.svg"));
        write_if_changed(&icon_path, ICON_SVG);
    }

    // b. pixmaps directory (direct fallback for desktop environments without theme cache)
    let pixmaps_dir = data_dir.join("pixmaps");
    if let Ok(()) = fs::create_dir_all(&pixmaps_dir) {
        let icon_path = pixmaps_dir.join(format!("{APP_ID}.svg"));
        write_if_changed(&icon_path, ICON_SVG);
    }

    // c. user icons root
    let user_icons = data_dir.join("icons");
    if let Ok(()) = fs::create_dir_all(&user_icons) {
        let icon_path = user_icons.join(format!("{APP_ID}.svg"));
        write_if_changed(&icon_path, ICON_SVG);
    }

    // 2. Refresh icon cache if gtk-update-icon-cache is available
    let hicolor_dir = data_dir.join("icons/hicolor");
    let _ = std::process::Command::new("gtk-update-icon-cache")
        .arg("-q")
        .arg("-t")
        .arg(&hicolor_dir)
        .spawn();

    // 3. Install .desktop file to ~/.local/share/applications/
    if let Ok(exe_path) = std::env::current_exe() {
        let apps_dir = data_dir.join("applications");
        if let Ok(()) = fs::create_dir_all(&apps_dir) {
            let desktop_path = apps_dir.join(format!("{APP_ID}.desktop"));
            let desktop_entry = format!(
                "[Desktop Entry]\n\
                 Type=Application\n\
                 Name=Nika's lib\n\
                 Comment=Personal book library manager\n\
                 Exec=\"{}\"\n\
                 Icon={}\n\
                 Terminal=false\n\
                 Categories=Office;Utility;\n\
                 StartupWMClass={}\n",
                exe_path.display(),
                APP_ID,
                APP_ID,
            );
            write_if_changed(&desktop_path, &desktop_entry);
        }
    }
}

pub fn register_icon_theme() {
    let data_dir = relm4::gtk::glib::user_data_dir();
    if let Some(display) = relm4::gtk::gdk::Display::default() {
        let theme = relm4::gtk::IconTheme::for_display(&display);
        theme.add_search_path(data_dir.join("icons"));
        theme.add_search_path(data_dir.join("icons/hicolor/scalable/apps"));
        theme.add_search_path(data_dir.join("pixmaps"));
    }
}

fn write_if_changed(path: &std::path::Path, content: &str) {
    let needs_write = match fs::read_to_string(path) {
        Ok(existing) => existing != content,
        Err(_) => true,
    };
    if needs_write {
        let _ = fs::write(path, content);
    }
}
