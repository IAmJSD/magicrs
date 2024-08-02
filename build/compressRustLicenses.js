"use strict";

const { third_party_libraries } = require("../rust_licenses.json");
const fs = require("fs");
const { join } = require("path");

const newLicensesArr = [];

const licenseMap = new Map();
for (const lib of third_party_libraries) {
    const license = lib.licenses[0];
    if (!license) continue;
    const index = licenseMap.get(license.text);
    if (index === undefined) {
        const nextIndex = newLicensesArr.length;
        licenseMap.set(license.text, nextIndex);
        newLicensesArr.push([
            lib.package_name, lib.package_version, lib.repository,
            license.text,
        ]);
    } else {
        newLicensesArr.push([
            lib.package_name, lib.package_version, lib.repository,
            index,
        ]);
    }
}

const compressedPath = join(__dirname, "..", "rust_licenses.compressed.json");

fs.writeFileSync(compressedPath, JSON.stringify({ libraries: newLicensesArr }));
