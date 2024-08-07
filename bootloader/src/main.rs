use base64::{prelude::BASE64_STANDARD, Engine};
#[cfg(feature = "signature")]
use openssl::{hash::MessageDigest, pkey::PKey, sign::Verifier};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn core_dylib() -> &'static [u8] {
    include_bytes!("../../target/release/libmagiccap_core.dylib")
}

#[cfg(target_os = "windows")]
fn core_dylib() -> &'static [u8] {
    include_bytes!("../../target/release/magiccap_core.dll")
}

#[cfg(target_os = "linux")]
fn core_dylib() -> &'static [u8] {
    include_bytes!("../../target/release/libmagiccap_core.so")
}

#[cfg(target_os = "macos")]
fn dll_ext() -> &'static str {
    "dylib"
}

#[cfg(target_os = "windows")]
fn dll_ext() -> &'static str {
    "dll"
}

#[cfg(target_os = "linux")]
fn dll_ext() -> &'static str {
    "so"
}

fn copy_application_core_bundle(
    binaries_dir: &PathBuf,
    core: &[u8],
    core_signature: Option<&[u8]>,
    application_bundle_hash: &str,
) {
    // Copy the core library into ~/.config/magiccap/binaries.
    let core_path = binaries_dir.join(format!("core.{}", dll_ext()));
    fs::write(&core_path, core).unwrap();

    // Copy the core library signature into ~/.config/magiccap/binaries.
    let core_signature_path = binaries_dir.join(format!("core.{}.sig", dll_ext()));
    if let Some(core_signature) = core_signature {
        fs::write(&core_signature_path, core_signature).unwrap();
    }

    // Copy the application bundle hash into ~/.config/magiccap/binaries.
    let application_hash_path = binaries_dir.join("core.application_hash.txt");
    fs::write(&application_hash_path, application_bundle_hash).unwrap();
}

#[cfg(feature = "signature")]
fn get_core_signature() -> Option<&'static [u8]> {
    Some(include_bytes!("../../target/release/magiccap_core.sig"))
}

#[cfg(not(feature = "signature"))]
fn get_core_signature() -> Option<&'static [u8]> {
    None
}

#[cfg(feature = "signature")]
fn core_sig_match(binaries_dir: &PathBuf, public_key: &str) -> bool {
    // Read the core signature.
    let core_signature = match fs::read(binaries_dir.join(format!("core.{}.sig", dll_ext()))) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Read the core library.
    let core = fs::read(binaries_dir.join(format!("core.{}", dll_ext()))).unwrap();

    // Load the public key.
    let public_key = PKey::public_key_from_pem(public_key.as_bytes()).unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key).unwrap();

    // Verify the signature.
    verifier.update(&core).unwrap();
    verifier.verify(&core_signature).unwrap()
}

#[cfg(not(feature = "signature"))]
fn core_sig_match(_binaries_dir: &PathBuf, _public_key: &str) -> bool {
    true
}

fn load_core(binaries_dir: &PathBuf) {
    unsafe {
        // Load the core library.
        let lib =
            libloading::Library::new(&binaries_dir.join(format!("core.{}", dll_ext()))).unwrap();

        // Load the application_init function.
        let application_init: libloading::Symbol<unsafe extern "C" fn()> =
            lib.get(b"application_init").unwrap();

        // Call the application_init function.
        application_init();
    }
}

fn main() {
    // Set MAGICCAP_INTERNAL_STARTED_WITH_BOOTLOADER to 1.
    std::env::set_var("MAGICCAP_INTERNAL_STARTED_WITH_BOOTLOADER", "1");

    // Include the core library.
    let core = core_dylib();

    // Create a sha256 of the core library we have. This is the "application bundle hash". If this changes,
    // then we know that the user has updated the application.
    let mut hasher = Sha256::new();
    hasher.update(core);
    let application_bundle_hash = BASE64_STANDARD.encode(hasher.finalize());

    // Make sure that ~/.config/magiccap/binaries exists.
    let homedir = home::home_dir().unwrap();
    let binaries_dir = homedir.join(".config").join("magiccap").join("binaries");
    fs::create_dir_all(&binaries_dir).unwrap();

    // Check if core.application_hash.txt is equal.
    let hashes_equal = match fs::read_to_string(binaries_dir.join("core.application_hash.txt")) {
        Ok(hash) => hash == application_bundle_hash,
        Err(_) => false,
    };

    // Get the signature of the core library built in.
    let core_signature = get_core_signature();

    // If it doesn't match, copy in the core library from the application bundle.
    if !hashes_equal {
        println!("[bootloader] Application bundle hash does not match. Copying in core library from bundle.");
        copy_application_core_bundle(
            &binaries_dir,
            core,
            core_signature,
            &application_bundle_hash,
        );
    }

    // Defines the public key for MagicCap. This is stored in the root of the project.
    let public_key = include_str!("../../build_signing.pub");
    if !core_sig_match(&binaries_dir, public_key) {
        println!(
            "[bootloader] Core signature does not match. Copying in core library from bundle."
        );
        copy_application_core_bundle(
            &binaries_dir,
            core,
            core_signature,
            &application_bundle_hash,
        );
    }

    // Load the core library.
    load_core(&binaries_dir);
}
