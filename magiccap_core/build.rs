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

#[cfg(not(target_os = "macos"))]
fn os_specific_compilation() {}

fn main() {
    // Handle OS-specific compilation.
    os_specific_compilation();
}
