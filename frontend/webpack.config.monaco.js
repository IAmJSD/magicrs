const MonacoWebpackPlugin = require("monaco-editor-webpack-plugin");
const path = require("path");

module.exports = {
	entry: "./src/monacoSetup.js",
    mode: "production",
	output: {
		path: path.resolve(__dirname, "public", "monaco"),
		filename: "monacoSetup.mjs",
        publicPath: "/monaco",
	},
    cache: true,
	devtool: false,
	module: {
		rules: [
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
