use tauri::image::Image;

/// Canonical app artwork (same file as bundle `256x256.png`).
pub fn app_icon_rgba() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/256x256.png")).expect("decode embedded app icon PNG")
}

/// Tray icon for menu bar/system tray.
pub fn tray_icon_rgba() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/tray-icon.png")).expect("decode embedded tray icon PNG")
}
