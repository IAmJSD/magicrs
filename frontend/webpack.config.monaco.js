const MonacoWebpackPlugin = require("monaco-editor-webpack-plugin");
const path = require("path");

module.exports = {
    entry: "./src/monacoSetup.js",
    mode: "production",
    output: {
        path: path.resolve(__dirname, "public", "monaco"),
        filename: (pathData) => {
            return pathData.chunk.name === "main" ? "monacoSetup.mjs" : "monacoSetup.[hash].mjs";
        },
        publicPath: "/monaco",
    },
    cache: true,
    devtool: false,
    module: {
        rules: [
            {
                loader: "webpack-query-loader",
                options: {
                    resourceQuery: "worker",
                    use: {
                        loader: "worker-rspack-loader",
                    },
                },
            },
            {
                test: /\.css$/,
                use: ["style-loader", "css-loader"],
            },
            {
                test: /\.ttf$/,
                type: "asset/resource",
            },
        ],
    },
    plugins: [new MonacoWebpackPlugin()],
};
