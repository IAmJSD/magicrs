"use strict";

const { get } = require("axios");
const { join } = require("path");
const { writeFileSync } = require("fs");
const AdmZip = require("adm-zip");
const crypto = require("crypto");

function hashZip(data) {
    // Load the zip file.
    const zip = new AdmZip(data);
    const zipEntries = zip.getEntries().sort((a, b) => {
        return a.entryName.localeCompare(b.entryName);
    });

    // Map into an array of [name, hash].
    const hashes = zipEntries.map(entry => {
        const hash = crypto.createHash("sha256");
        hash.update(entry.getData());
        return [entry.entryName, hash.digest("hex")];
    });

    // Create a hash of the hashes.
    const hash = crypto.createHash("sha256");
    hash.update(JSON.stringify(hashes));
    return hash.digest("hex");
}

async function main() {
    // Get the releases from the GitHub releases.
    const { data } = await get(
        "https://api.github.com/repos/WebScaleSoftwareLtd/magiccap-php-binaries/releases",
    );
    if (!Array.isArray(data) || data.length === 0) {
        throw new Error("No releases found.");
    }
    const release = data[0];

    // Get the PHP Windows releases.
    const { data: phpReleases } = await get(
        "https://windows.php.net/downloads/releases/releases.json",
    );
    let phpRelease = null;
    for (const releaseKey of Object.keys(phpReleases)) {
        // Check if the name is with the same version.
        if (release.name.startsWith(releaseKey)) {
            phpRelease = phpReleases[releaseKey];
            break;
        }
    }
    if (!phpRelease) {
        throw new Error("No PHP Windows release found.");
    }

    // Get the releases from GitHub.
    const releases = {};
    const promises = [];
    for (const asset of release.assets) {
        promises.push((async () => {
            // Get the name and URL of the asset.
            const name = asset.name;
            const url = asset.browser_download_url;

            // Get the asset URL.
            const { data } = await get(url, {
                responseType: "arraybuffer",
            });

            // Create a hash of the asset.
            const hash = crypto.createHash("sha256");
            hash.update(data);
            const sha256 = hash.digest("hex");

            // Add the asset to the releases.
            releases[name] = {
                url,
                sha256,
                zip: false,
            };
        })());
    }

    // Get the nts-vc15-x64 version.
    const ntsVs16X64 = phpRelease["nts-vs16-x64"];
    if (!ntsVs16X64) {
        throw new Error("No nts-vc16-x64 version found.");
    }

    // Check if there is a arm64 version.
    const arm64 = phpRelease["nts-vs16-arm64"];

    // Handle the promise to get the x64 version.
    promises.push((async () => {
        // Get the asset URL.
        const url = `https://windows.php.net/downloads/releases/${ntsVs16X64.zip.path}`;
        const { data } = await get(url, {
            responseType: "arraybuffer",
        });

        // Create the info blob.
        const info = {
            url,
            sha256: hashZip(data),
            zip: true,
        };

        // Add the asset to the releases.
        releases["php-windows-x86_64"] = info;
        if (!arm64) {
            // There is no ARM build meaning that ARM windows systems will need to emulate x64.
            releases["php-windows-arm64"] = info;
        }
    })());

    // Handle the promise to get the arm64 version.
    if (arm64) {
        promises.push((async () => {
            // Get the asset URL.
            const url = `https://windows.php.net/downloads/releases/${arm64.zip.path}`;
            const { data } = await get(url, {
                responseType: "arraybuffer",
            });

            // Add the asset to the releases.
            releases["php-windows-arm64"] = {
                url,
                sha256: hashZip(data),
                zip: true,
            };
        })());
    }

    // Write the artifact map to a file.
    await Promise.all(promises);
    const path = join(
        __dirname, "..", "..", "php_artifacts.json",
    );
    writeFileSync(path, JSON.stringify(releases, null, 4) + "\n");
    console.log(`PHP version updated to ${release.name}`);
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
