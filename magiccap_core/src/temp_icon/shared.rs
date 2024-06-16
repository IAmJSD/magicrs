use once_cell::sync::Lazy;
use tray_icon::Icon;

// Handle creating the stop icon.
pub static STOP_ICON: Lazy<Icon> = Lazy::new(|| {
    // Get the png file.
    static STOP_PNG: &[u8] = include_bytes!("../../../assets/stop.png");

    // Get the rgba data.
    let rgba = image::load_from_memory(STOP_PNG).unwrap().to_rgba8();

    // Return the Icon.
    Icon::from_rgba(rgba.to_vec(), rgba.width(), rgba.height()).unwrap()
});

// Handle creating the cog icon.
pub static COG_ICON: Lazy<Icon> = Lazy::new(|| {
    // Get the png file.
    static COG_PNG: &[u8] = include_bytes!("../../../assets/cog.png");

    // Get the rgba data.
    let rgba = image::load_from_memory(COG_PNG).unwrap().to_rgba8();

    // Return the Icon.
    Icon::from_rgba(rgba.to_vec(), rgba.width(), rgba.height()).unwrap()
});
