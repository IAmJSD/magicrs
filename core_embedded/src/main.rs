use std::fs;

fn main() {
    // Make sure that ~/.config/magiccap/binaries exists. The bootloader does this too, but we do not have
    // access to the bootloader here.
    let homedir = home::home_dir().unwrap();
    let binaries_dir = homedir.join(".config").join("magiccap").join("binaries");
    fs::create_dir_all(&binaries_dir).unwrap();

    // Go ahead and load the core library.
    unsafe { magiccap_core::application_init() }
}
