"use strict";

const { join } = require("path");
const { execSync } = require("child_process");

// Run the specified command.
function runCommand(command) {
    // Log the command we're running.
    console.log(`\x1b[1m\x1b[37m$\x1b[0m ${command}`);

    try {
        // Run the command and log the output.
        execSync(command, {
            env: process.env,
            shell: process.env.SHELL || true,
            stdio: "inherit",
            cwd: join(__dirname, ".."),
        });
    } catch (e) {
        // If we crash due to a error code, just exit with the same code.
        if (e.status) {
            process.exit(e.status);
        }

        // Otherwise, something else went wrong. Just log the error and exit.
        console.error(e);
        process.exit(1);
    }
}

// Compile without autoupdate on macOS.
function macOSNoAutoupdateCompilation() {
    // Make sure both Intel and Apple Silicon targets are available.
    runCommand("rustup target add x86_64-apple-darwin");
    runCommand("rustup target add aarch64-apple-darwin");

    // Compile the rest of the project on Intel.
    runCommand("cargo build --release --package core_embedded --target x86_64-apple-darwin");

    // Compile the rest of the project on Apple Silicon.
    runCommand("cargo build --release --package core_embedded --target aarch64-apple-darwin");

    // Create a universal binary.
    runCommand("lipo -create -output target/release/core_embedded target/release/core_embedded-x86_64-apple-darwin target/release/core_embedded-aarch64-apple-darwin");

    // Return the path to the compiled binary.
    return join(__dirname, "..", "target", "release", "core_embedded");
}

// Build without autoupdate.
function buildNoAutoupdate() {
    // Generate the Rust licenses file.
    runCommand("make generate-license-file");

    // Compile the frontend.
    runCommand("cd frontend && npm ci && npm run build");

    // Download the models.
    runCommand("cd build/download-models && npm ci && node .");

    // Handle macOS compilation.
    if (process.platform === "darwin") {
        return macOSNoAutoupdateCompilation();
    }

    // Compile the rest of the project.
    runCommand("cargo build --release --package core_embedded");

    // Return the path to the compiled binary.
    let bin = "core_embedded";
    if (process.platform === "win32") bin += ".exe";
    return join(__dirname, "..", "target", "release", bin);
}

// Defines the MagicCap Core file path.
let corePath;
switch (process.platform) {
    case "darwin":
        corePath = join(__dirname, "..", "target", "release", "libmagiccap_core.dylib");
        break;
    case "win32":
        corePath = join(__dirname, "..", "target", "release", "magiccap_core.dll");
        break;
    case "linux":
        corePath = join(__dirname, "..", "target", "release", "libmagiccap_core.so");
        break;
    default:
        console.error("Unsupported platform.");
        process.exit(1);
}

// Defines the MagicCap Core signature file path.
const sigPath = join(__dirname, "..", "target", "release", "magiccap_core.sig");

// Compile MagicCap Core with autoupdate on macOS.
function macOSAutoupdateCompilation(privateKeyPath) {
    // Make sure both Intel and Apple Silicon targets are available.
    runCommand("rustup target add x86_64-apple-darwin");
    runCommand("rustup target add aarch64-apple-darwin");

    // Compile the rest of the project on Intel.
    runCommand("cargo build --release --package magiccap_core --features signature --target x86_64-apple-darwin");

    // Compile the rest of the project on Apple Silicon.
    runCommand("cargo build --release --package magiccap_core --features signature --target aarch64-apple-darwin");

    // Copy the Intel library and sign it.
    runCommand(`cp target/x86_64-apple-darwin/release/libmagiccap_core.dylib "${corePath}"`);
    runCommand(`openssl dgst -sha256 -sign "${privateKeyPath}" -out "${sigPath}" "${corePath}"`);

    // Compile the bootloader for Intel.
    runCommand("cargo build --release --package bootloader --features signature --target x86_64-apple-darwin");

    // Copy the signature to dist/core_x86.sig.
    runCommand(`rm -rf dist && mkdir dist && cp "${sigPath}" dist/core_x86.sig`);

    // Copy the library.
    runCommand(`cp "${corePath}" dist/core_x86.dylib`);

    // Copy the Apple Silicon library and sign it.
    runCommand(`cp target/aarch64-apple-darwin/release/libmagiccap_core.dylib "${corePath}"`);
    runCommand(`openssl dgst -sha256 -sign "${privateKeyPath}" -out "${sigPath}" "${corePath}"`);

    // Compile the bootloader for Apple Silicon.
    runCommand("cargo build --release --package bootloader --features signature --target aarch64-apple-darwin");

    // Copy the signature to dist/core_arm.sig.
    runCommand(`cp "${sigPath}" dist/core_arm.sig && rm "${sigPath}"`);

    // Copy the library.
    runCommand(`cp "${corePath}" dist/core_arm.dylib && rm "${corePath}"`);

    // Create a universal binary for the bootloader .
    runCommand("lipo -create -output target/release/bootloader target/release/bootloader-x86_64-apple-darwin target/release/bootloader-aarch64-apple-darwin");

    // Return the path to the compiled binary.
    return join(__dirname, "..", "target", "release", "bootloader");
}

// Build with autoupdate.
function buildWithAutoupdate(privateKeyPath) {
    // Check that build_signing.pub is the public key for the private key at the given path.
    runCommand(`openssl rsa -in "${privateKeyPath}" -pubout -outform PEM | diff - build_signing.pub`);

    // Generate the Rust licenses file.
    runCommand("make generate-license-file");

    // Compile the frontend.
    runCommand("cd frontend && npm ci && npm run build");

    // Download the models.
    runCommand("cd build/download-models && npm ci && node .");

    if (process.platform === "darwin") {
        // Due to universal binaries, we need to compile differently on macOS.
        return macOSAutoupdateCompilation(privateKeyPath);
    }

    // Compile MagicCap core with the signature feature enabled.
    runCommand("cargo build --release --package magiccap_core --features signature");

    // Sign MagicCap Core.
    runCommand(`openssl dgst -sha256 -sign "${privateKeyPath}" -out "${sigPath}" "${corePath}"`);

    // Compile the bootloader to bootstrap the booting and signature verification process.
    runCommand("cargo build --release --package bootloader --features signature");

    // Move the signature and library to the dist folder.
    runCommand(`rm -rf dist && mkdir dist && mv "${sigPath}" dist/core.sig`);
    runCommand(`mv "${corePath}" dist/core${process.platform === "win32" ? ".dll" : ".so"}`);

    // Return the path to the compiled binary.
    let bin = "bootloader";
    if (process.platform === "win32") bin += ".exe";
    return join(__dirname, "..", "target", "release", bin);
}

// Compile MagicCap depending on the env variables set.
let binary;
if (process.env.MAGICCAP_AUTOUPDATE_PRIVATE_KEY) {
    console.log("Compiling MagicCap with autoupdate enabled.");
    binary = buildWithAutoupdate(process.env.MAGICCAP_AUTOUPDATE_PRIVATE_KEY);
} else {
    console.log("Compiling MagicCap without autoupdate.");
    binary = buildNoAutoupdate();

    // Clean the dist folder.
    runCommand("rm -rf dist && mkdir dist");
}

// Copy the compiled binary to the dist folder.
runCommand(`cp "${binary}" dist/magiccap${process.platform === "win32" ? ".exe" : ""}`);

// Send a success message.
const message = `\n\x1b[1mMagicCap has been successfully compiled!\x1b[0m The binary outputs for your platform are in the 'dist' folder.

Note that you may want to run \x1b[1m\x1b[37mmake package\x1b[0m to turn the binary into a installable package for your platform.
`;
console.log(message);
