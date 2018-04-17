var path = require('path');

module.exports = {
  mode: 'development',
  entry: './web/index.js',
  output: {
    path: path.resolve(__dirname, 'web'),
    filename: 'index.bundle.js'
  }
};
