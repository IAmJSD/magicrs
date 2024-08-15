#[cfg(target_os = "macos")]
fn os_specific_compilation() {
    // Watch for changes to the macOS Objective-C files.
    println!("cargo:rerun-if-changed=src/macos/macos.m");
    println!("cargo:rerun-if-changed=src/macos/macos.h");

    // Link in UserNotifications.
    println!("cargo:rustc-link-lib=framework=UserNotifications");

    // Compile the macOS Objective-C file.
    cc::Build::new().file("src/macos/macos.m").compile("macos");
}

#[cfg(target_os = "linux")]
fn os_specific_compilation() {
    // Watch for changes to linux_x11.c.
    println!("cargo:rerun-if-changed=src/region_selector/linux_x11.c");
    println!("cargo:rerun-if-changed=src/video_capture/linux_recorder.c");

    // Link in X11.
    println!("cargo:rustc-link-lib=X11");

    // Compile the Linux X11 file.
    cc::Build::new()
        .file("src/region_selector/linux_x11.c")
        .compile("linux_x11");

    // Compile the Linux recorder file.
    cc::Build::new()
        .file("src/video_capture/linux_recorder.c")
        .compile("linux_recorder");
}

#[cfg(target_os = "windows")]
fn os_specific_compilation() {}

fn main() {
    // Handle OS-specific compilation.
    os_specific_compilation();

    // Get the build timestamp.
    let build_timestamp = chrono::Utc::now().timestamp();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);

    // Get the Git commit hash and branch.
    let git_hash = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to get Git commit hash.")
        .stdout;
    let git_branch = std::process::Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get Git branch.")
        .stdout;
    println!(
        "cargo:rustc-env=GIT_HASH={}",
        String::from_utf8(git_hash).unwrap()
    );
    println!(
        "cargo:rustc-env=GIT_BRANCH={}",
        String::from_utf8(git_branch).unwrap()
    );
}
