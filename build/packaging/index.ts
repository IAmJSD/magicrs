import { join } from "path";
import { existsSync } from "fs";
import { execSync } from "child_process";

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

// Handle packaging macOS into a app/dmg.
function packageMacOS() {
    
}
