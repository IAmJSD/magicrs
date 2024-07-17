#[cfg(target_os = "windows")]
fn os_specific_compilation() {
    use embed_manifest::{embed_manifest, new_manifest};

    embed_manifest(new_manifest("MagicCap")).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(not(target_os = "windows"))]
fn os_specific_compilation() {}

fn main() {
    // Handle OS-specific compilation.
    os_specific_compilation();
}
