"use strict";

const { execSync } = require("child_process");
const { join } = require("path");

function runCommand(command) {
    // Log the command we're running.
    console.log(`\x1b[1m\x1b[37m$\x1b[0m ${command}`);

    try {
        // Run the command and log the output.
        execSync(command, {
            env: process.env,
            shell: process.env.SHELL || true,
            stdio: "inherit",
            cwd: __dirname,
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

// Build without autoupdate.
function buildNoAutoupdate() {
    // Generate the Rust licenses file.
    runCommand("make generate-license-file");

    // Compile the frontend.
    runCommand("cd frontend && yarn && yarn run build");

    // Compile the rest of the project.
    runCommand("cargo build --release --package core_embedded");

    // Return the path to the compiled binary.
    let bin = "core_embedded";
    if (process.platform === "win32") bin += ".exe";
    return join(__dirname, "target", "release", bin);
}

// Defines the MagicCap Core file path.
let corePath;
switch (process.platform) {
    case "darwin":
        corePath = join(__dirname, "target", "release", "libmagiccap_core.dylib");
        break;
    case "win32":
        corePath = join(__dirname, "target", "release", "magiccap_core.dll");
        break;
    case "linux":
        corePath = join(__dirname, "target", "release", "libmagiccap_core.so");
        break;
    default:
        console.error("Unsupported platform.");
        process.exit(1);
}

// Defines the MagicCap Core signature file path.
const sigPath = join(__dirname, "target", "release", "magiccap_core.sig");

// Build with autoupdate.
function buildWithAutoupdate(privateKeyPath) {
    // Check that build_signing.pub is the public key for the private key at the given path.
    runCommand(`openssl rsa -in "${privateKeyPath}" -pubout -outform PEM | diff - build_signing.pub`);

    // Generate the Rust licenses file.
    runCommand("make generate-license-file");

    // Compile the frontend.
    runCommand("cd frontend && yarn && yarn run build");

    // Compile MagicCap core with the signature feature enabled.
    runCommand("cargo build --release --package magiccap_core --features signature");

    // Sign MagicCap Core.
    runCommand(`openssl dgst -sha256 -sign "${privateKeyPath}" -out "${sigPath}" "${corePath}"`);

    // Compile the bootloader to bootstrap the booting and signature verification process.
    runCommand("cargo build --release --package bootloader --features signature");

    // Return the path to the compiled binary.
    let bin = "bootloader";
    if (process.platform === "win32") bin += ".exe";
    return join(__dirname, "target", "release", bin);
}

// Compile MagicCap depending on the env variables set.
let binary;
if (process.env.MAGICCAP_AUTOUPDATE_PRIVATE_KEY) {
    console.log("Compiling MagicCap with autoupdate enabled.");
    binary = buildWithAutoupdate(process.env.MAGICCAP_AUTOUPDATE_PRIVATE_KEY);
} else {
    console.log("Compiling MagicCap without autoupdate.");
    binary = buildNoAutoupdate();
}

// Copy the compiled binary to the dist folder.
runCommand(`rm -rf dist && mkdir dist && cp "${binary}" dist/magiccap${process.platform === "win32" ? ".exe" : ""}`);
if (process.env.MAGICCAP_AUTOUPDATE_PRIVATE_KEY) {
    let cmdStart;
    switch (process.platform) {
        case "darwin":
            cmdStart = `cp "${corePath}" dist/core.dylib && `;
            break;
        case "win32":
            cmdStart = `cp "${corePath}" dist/core.dll && `;
            break;
        case "linux":
            cmdStart = `cp "${corePath}" dist/core.so && `;
            break;
        default:
            console.error("Unsupported platform.");
            process.exit(1);
    }
    runCommand(cmdStart + `cp "${sigPath}" dist/core.sig`);
}

// Send a success message.
const message = `\n\x1b[1mMagicCap has been successfully compiled!\x1b[0m The binary outputs for your platform are in the 'dist' folder.

Note that you may want to run \x1b[1m\x1b[37mmake package\x1b[0m to turn the binary into a installable package for your platform.`;
console.log(message);
