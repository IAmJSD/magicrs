use std::{env, fs};

fn main() {
    // Set MAGICCAP_INTERNAL_STARTED_WITH_BOOTLOADER to 0.
    env::set_var("MAGICCAP_INTERNAL_STARTED_WITH_BOOTLOADER", "0");

    // Make sure that ~/.config/magiccap/binaries exists. The bootloader does this too, but we do not have
    // access to the bootloader here.
    let homedir = home::home_dir().unwrap();
    let binaries_dir = homedir.join(".config").join("magiccap").join("binaries");
    fs::create_dir_all(&binaries_dir).unwrap();

    // Go ahead and load the core library.
    unsafe { magiccap_core::application_init() }
}
