const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const FlowWebpackPlugin = require("flow-webpack-plugin");
const webpack = require("webpack");

module.exports = {
  mode: "production",
  entry: { 
    index: "./js/index.js",
    polyfills: "./js/polyfills.js",
  },
  output: {
    path: path.resolve(__dirname, "docs"),
    filename: '[name].bundle.js'
  },
  resolve: {
    modules: [__dirname, "node_modules"]
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
        }
      },
      {
        test: /\.css$/,
        use: ["style-loader", "css-loader"]
      }
    ]
  },
  plugins: [
    new HtmlWebpackPlugin({
      title: "Snake WASM",
      inject: "body",
      hash: true,
      template: "index.html",
      chunks: ["index"],
    }),
    // Have this example work in Edge which doesn't ship `TextEncoder` or
    // `TextDecoder` at this time.
    /*
     * new webpack.ProvidePlugin({
     *   TextDecoder: ["text-encoding", "TextDecoder"],
     *   TextEncoder: ["text-encoding", "TextEncoder"]
     * })
     */
  ],
};
