import plainText from "vite-plugin-plain-text";
import react from "@vitejs/plugin-react";
import license from "rollup-plugin-license";
import { join } from "path";

function remapDependencies(dependencies) {
    return dependencies.map(body => ({
        name: body.name,
        version: body.version,
        repo: body.repository,
        text: body.licenseText,
    }));
}

export default {
    plugins: [
        plainText([/\.md$/]),
        react(),
    ],
    build: {
        chunkSizeWarningLimit: 1000,
        rollupOptions: {
            plugins: [
                license({
                    thirdParty: {
                        output: {
                            file: join(__dirname, "dist", "js_licenses.json"),
                            template(dependencies) {
                                return JSON.stringify(remapDependencies(dependencies));
                            },
                        },
                    },
                }),
            ],
        },
    },
};
