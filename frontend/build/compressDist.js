"use strict";

const { readdirSync } = require("fs");
const { join } = require("path");
const { c } = require("tar");

const distFolder = join(__dirname, "..", "dist");
const files = readdirSync(distFolder);
c({
    z: true,
    f: join(distFolder, "dist.tgz"),
    cwd: distFolder,
}, files).then(() => {
    console.log("Compressed dist folder.");
});
