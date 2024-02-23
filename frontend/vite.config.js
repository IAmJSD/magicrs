import plainText from "vite-plugin-plain-text";

export default {
    plugins: [plainText([/\.md$/])],
};
