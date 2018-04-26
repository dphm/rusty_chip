var path = require('path');

module.exports = {
  mode: 'development',
  entry: './bootstrap.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bootstrap.js'
  }
};
