const { config } = require("@swc/core/spack");

module.exports = config({
  entry: {
    web: __dirname + "/src/main.ts",
  },
  output: {
    path: __dirname + "/dist",
    name: "main.js",
  },
  options: {
    minify: true,
  },
});
