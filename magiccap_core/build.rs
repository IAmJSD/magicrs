#[cfg(target_os = "macos")]
fn os_specific_compilation() {
    // Watch for changes to the macOS Objective-C files.
    println!("cargo:rerun-if-changed=src/macos/macos.m");
    println!("cargo:rerun-if-changed=src/macos/macos.h");

    // Link in UserNotifications.
    println!("cargo:rustc-link-lib=framework=UserNotifications");

    // Compile the macOS Objective-C file.
    cc::Build::new()
        .file("src/macos/macos.m")
        .compile("macos");
}

#[cfg(target_os = "linux")]
fn os_specific_compilation() {
    // Watch for changes to linux_x11.c.
    println!("cargo:rerun-if-changed=src/region_selector/linux_x11.c");

    // Link in X11.
    println!("cargo:rustc-link-lib=X11");

    // Compile the Linux X11 file.
    cc::Build::new()
        .file("src/region_selector/linux_x11.c")
        .compile("linux_x11");
}

fn main() {
    // Handle OS-specific compilation.
    os_specific_compilation();
}
