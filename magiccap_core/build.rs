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

fn compile_region_selector() {
    // Watch for changes to the region selector files.
    println!("cargo:rerun-if-changed=src/region_selector/engine.c");
    println!("cargo:rerun-if-changed=src/region_selector/engine.m");

    // Link in OpenGL.
    println!("cargo:rustc-link-lib=framework=OpenGL");

    // Find the GLFW3 library.
    let glfw3 = pkg_config::Config::new()
        .atleast_version("3.3")
        .probe("glfw3")
        .unwrap();

    // Link in the GLFW3 library.
    for path in glfw3.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    // Compile the region selector engine.
    cc::Build::new()
        .file("src/region_selector/engine.c")
        .includes(glfw3.include_paths)
        .static_flag(true)
        .debug(true)
        .compile("region_selector");
}

fn main() {
    // Handle OS-specific compilation.
    os_specific_compilation();

    // Handle compiling the region selector.
    compile_region_selector();
}
