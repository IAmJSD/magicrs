// Defines the type for license data.
export type LicenseData = {
    name: string;
    version: string;
    repo: string;
    text: string;
};

// Handles iteration of Rust library licenses.
export class RustLibraryLicenseIterator {
    constructor(private libraries: (string | number)[][]) { }

    get(index: number): LicenseData {
        const [name, version, repo, textOrIndex] = this.libraries[index];
        const text = typeof textOrIndex === "number" ? this.libraries[textOrIndex][3] as string : textOrIndex;
        return {
            name: name as string,
            version: version as string,
            repo: repo as string,
            text,
        };
    }

    *[Symbol.iterator]() {
        for (let i = 0; i < this.libraries.length; i++) {
            yield this.get(i);
        }
    }

    get length() {
        return this.libraries.length;
    }
}

// Import in the Rust licenses.
export const rustLicenses = import("../../rust_licenses.compressed.json").then(x => new RustLibraryLicenseIterator(x.libraries));

// Import in the JS licenses.
export const jsLicenses = fetch("/js_licenses.json").then(x => {
    if (!x.ok) {
        throw new Error("Failed to fetch JS licenses.");
    }
    return x.json() as Promise<LicenseData[]>;
});
