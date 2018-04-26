const path = require("path");

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  module: {
    rules: [
      {
        test: /\.ch8$/,
        use: 'bin-loader'
      }
    ]
  },
  mode: "development"
};

