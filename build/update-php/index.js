"use strict";

const { get } = require("axios");
const { join } = require("path");
const { writeFileSync } = require("fs");
const AdmZip = require("adm-zip");
const crypto = require("crypto");

const PREFIX = /^php-(\d+\.\d+)/;

async function processArtifact(artifact, runId) {
    // Build the download URL.
    const downloadUrl = "https://github.com/static-php/static-php-cli-hosted/actions/runs/" + runId + "/artifacts/" + artifact.id;

    // Get either 'php' or 'php.exe' from the ZIP.
    const zip = new AdmZip(data);
    const zipEntries = zip.getEntries();
    const entry = zipEntries.find(entry => entry.entryName.endsWith("php") || entry.entryName.endsWith("php.exe"));
    if (!entry) {
        throw new Error("Could not find PHP binary in artifact.");
    }

    // Extract the PHP binary as a buffer.
    const buffer = entry.getData();

    // sha256 the buffer.
    const hash = crypto.createHash("sha256");
    hash.update(buffer);
    const sha256 = hash.digest("hex");

    // Return the URL and hash.
    return {
        url: artifact.archive_download_url,
        hash: sha256,
    };
}

async function main() {
    // Get all the workflow runs.
    const { data } = await get("https://api.github.com/repos/static-php/static-php-cli-hosted/actions/workflows/77464426/runs", {
        responseType: "json",
    });

    // Find the first successful run.
    const successfulRun = data.workflow_runs.find(run => run.conclusion === "success");

    // Visit the artifacts_url of the successful run.
    const artifacts = (await get(successfulRun.artifacts_url, {
        responseType: "json",
    })).data.artifacts;

    // Reverse the artifacts and then break when we find the next PHP version.
    const latestVersionArtifacts = [];
    const lastVersion = artifacts[artifacts.length - 1].name.match(PREFIX)[1];
    for (const artifact of artifacts.reverse()) {
        const version = artifact.name.match(PREFIX)[1];
        if (version !== lastVersion) {
            break;
        }
        latestVersionArtifacts.push(artifact);
    }

    // Defines the map and then handle the artifacts.
    const artifactMap = {};
    const promises = artifacts.map(artifact => (async () => {
        const res = await processArtifact(artifact);
        artifactMap[artifact.name] = res;
    })());
    await Promise.all(promises);

    // Write the artifact map to a file.
    const path = join(
        __dirname, "..", "..", "php_artifacts.json",
    );
    writeFileSync(path, JSON.stringify(artifactMap, null, 4));
    console.log(`PHP version updated to ${lastVersion}`);
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
