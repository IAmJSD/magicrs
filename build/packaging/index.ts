import { join } from "path";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { execSync } from "child_process";
import createDMG from "electron-installer-dmg";

// Defines the root of the project.
const root = join(__dirname, "..", "..");

// Make sure the binary exists.
const binary = join(root, "dist", "magiccap");
if (!existsSync(binary)) {
    console.error("MagicCap binary not found. Make sure to build the project first.");
    process.exit(1);
}

// Get the number of commits in main.
const commitCount = Number(execSync("git rev-list --count main").toString().trim());
if (isNaN(commitCount)) {
    console.error("Failed to get commit count with git.");
    process.exit(1);
}

// Since MagicCap doesn't follow semver and every release from now on is under 3.x,
// we can just use the commit count as the minor version.
const version = `3.${commitCount}.0`;

// Log that we are packaging the project.
console.log(`Packaging MagicCap with version ${version}`);

// Run the specified command.
function runCommand(command: string) {
    // Log the command we're running.
    console.log(`\x1b[1m\x1b[37m$\x1b[0m ${command}`);

    try {
        // Run the command and log the output.
        execSync(command, {
            env: process.env,
            shell: process.env.SHELL,
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

// Handle packaging macOS into a app/dmg.
function packageMacOS() {
    // Read macos/info.plist.tmpl.
    let infoPlist = readFileSync(join(root, "macos", "Info.plist.tmpl"), { encoding: "utf8" });

    // Replace the version in the template.
    infoPlist = infoPlist.replaceAll("{version}", version);

    // Write the application structure.
    mkdirSync(
        join(root, "dist", "MagicCap.app", "Contents", "MacOS"),
        { recursive: true },
    );

    // Copy the binary to the application.
    copyFileSync(
        binary,
        join(root, "dist", "MagicCap.app", "Contents", "MacOS", "MagicCap"),
    );

    // Write the Info.plist.
    writeFileSync(
        join(root, "dist", "MagicCap.app", "Contents", "Info.plist"),
        infoPlist,
    );

    // Copy the icon to the resources folder.
    mkdirSync(
        join(root, "dist", "MagicCap.app", "Contents", "Resources"),
        { recursive: true },
    );
    copyFileSync(
        join(root, "assets", "icon.icns"),
        join(root, "dist", "MagicCap.app", "Contents", "Resources", "icon.icns"),
    );

    // Check if we should sign the application.
    if (process.env.MAGICCAP_MACOS_KEY_NAME) {
        console.log("Signing the application with the provided key.");
        runCommand(`codesign --deep --force --verbose --sign "${process.env.MAGICCAP_MACOS_KEY_NAME}" "${join(root, "dist", "MagicCap.app")}"`);
    }

    // Create the dmg.
    const codeSign = process.env.MAGICCAP_MACOS_KEY_NAME ? {
        "signing-identity": process.env.MAGICCAP_MACOS_KEY_NAME,
    } : undefined;
    createDMG({
        appPath: join(root, "dist", "MagicCap.app"),
        name: "MagicCap",
        out: join(root, "dist", "magiccap.dmg"),
        additionalDMGOptions: {
            "code-sign": codeSign,
        },
    }).then(() => {
        console.log("Application successfully packaged.");
    }).catch((e: any) => {
        console.error(e);
        process.exit(1);
    });
}

// Handle packaging GNU/Linux into its specific formats.
function packageGnuLinux() {
    // TODO
}

// Switch on the platform.
switch (process.platform) {
    case "darwin":
        packageMacOS();
        break;
    case "linux":
        packageGnuLinux();
        break;
    default:
        console.error("Unsupported platform.");
        process.exit(1);
}
