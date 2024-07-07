"use strict";

const { mkdirSync, rmSync } = require("fs");
const { writeFile } = require("fs/promises");
const { join } = require("path");
const { get } = require("axios");
const { gzip } = require("zlib");

const distFolder = join(__dirname, "dist");
try {
    rmSync(distFolder, { recursive: true });
} catch {}
mkdirSync(distFolder);

const models = [
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten",
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten",
];

function gzipPromise(data) {
    return new Promise((resolve, reject) => {
        gzip(data, { level: 9 }, (e, r) => {
            if (e) {
                reject(e);
            } else {
                resolve(r);
            }
        });
    });
}

for (const model of models) {
    get(model, { responseType: "arraybuffer" }).then(async r => {
        // Check if the response is ok.
        if (r.status !== 200) {
            throw new Error(`Failed to download model: ${model}`);
        }

        // Write the compressed model to disk.
        const modelPath = join(distFolder, model.split("/").pop() + ".gz");
        const compressed = await gzipPromise(r.data);
        await writeFile(modelPath, compressed);
        console.log(`Downloaded and compressed model: ${model}`);
    }).catch(e => {
        console.error(e);
        process.exit(1);
    });
}
