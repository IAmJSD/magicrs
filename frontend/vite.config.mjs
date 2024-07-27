import plainText from "vite-plugin-plain-text";
import react from "@vitejs/plugin-react";

export default {
    plugins: [
        plainText([/\.md$/]),
        react(),
    ],
};
