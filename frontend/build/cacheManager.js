"use strict";

const { readFile } = require("fs/promises");
const { dependencies } = require("../package.json");
const { createHash } = require("crypto");
const { writeFileSync, readFileSync } = require("fs");
const { join } = require("path");

// Defines the promise to get the new cache hash.
const newCacheHash = Promise.all([
    readFile(join(__dirname, "..", "webpack.config.monaco.js"), { encoding: "utf-8" }),
    readFile(join(__dirname, "..", "src", "monacoSetup.js"), { encoding: "utf-8" }),
    Promise.resolve(dependencies["monaco-editor"]),
]).then(a => {
    const hasher = createHash("sha256");
    hasher.write(JSON.stringify(a));
    return hasher.digest().toString("hex");
});

// Handle if the argument is 'set'.
const keyPath = join(__dirname, "..", "public", "monaco", "cache_key.txt");
if (process.argv[2] === "set") {
    // Write the cache key.
    newCacheHash.then(v => {
        writeFileSync(keyPath, v);
        console.log("Cache key successfully updated.");
    }).catch(err => {
        console.error(err);
        process.exit(1);
    });
} else {
    // Get the current cache hash.
    let cacheHash;
    try {
        cacheHash = readFileSync(keyPath, {
            encoding: "utf-8",
        });
    } catch {
        console.error("No cached build currently exists. Building from scratch...");
        process.exit(1);
    }

    // Error if they are different.
    newCacheHash.then(v => {
        if (v === cacheHash) {
            console.log("Build already ran.");
        } else {
            console.error("Hash different. Running build.");
            process.exit(1);
        }
    }).catch(err => {
        console.error(err);
        process.exit(1);
    });
}
