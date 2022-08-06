import path from 'path';
import webpack from 'webpack';
import 'webpack-dev-server';

import CopyPlugin from 'copy-webpack-plugin';
import MiniCssExtractPlugin from 'mini-css-extract-plugin';
import MinifyHtmlWebpackPlugin from 'minify-html-webpack-plugin';
import SveltePreprocess from 'svelte-preprocess';

const is_prod = process.env.NODE_ENV === 'production';
const mode = is_prod ? 'production' : 'development';

const config: webpack.Configuration = {
  entry: {
    main: './src/AppRestreamer.ts',
    'mix/main': './src/AppMix.ts',
    'dashboard/main': './src/AppDashboard.ts',
  },
  resolve: {
    alias: {
      svelte: path.resolve('node_modules', 'svelte'),
    },
    extensions: ['.mjs', '.js', '.ts', '.svelte'],
    mainFields: ['svelte', 'browser', 'module', 'main'],
  },
  output: {
    path: __dirname + '/public',
    filename: '[name].js',
    chunkFilename: '[name].[id].js',
  },
  devServer: {
    static: path.join(__dirname, 'public'),
    compress: true,
    port: 8080,
    host: '0.0.0.0',
  },
  module: {
    rules: [
      {
        test: /\.m?js/,
        type: 'javascript/auto',
      },
      {
        test: /\.m?js/,
        resolve: {
          fullySpecified: false,
        },
      },
      {
        test: /\.svelte$/,
        use: {
          loader: 'svelte-loader',
          options: {
            preprocess: SveltePreprocess({}),
            emitCss: true,
            hotReload: true,
          },
        },
      },
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: 'ts-loader',
      },
      {
        test: /\.css$/,
        use: [
          // 'mini-css-extract-plugin' doesn't support HMR.
          // Use 'style-loader' instead for development.
          is_prod ? MiniCssExtractPlugin.loader : 'style-loader',
          'css-loader',
        ],
      },
      {
        test: /\.graphql$/,
        exclude: /node_modules/,
        use: 'graphql-tag/loader',
      },
    ],
  },
  mode,
  plugins: [
    new CopyPlugin({
      patterns: [
        { from: 'static/index.html' },
        { from: 'static/mix', to: 'mix' },
        { from: 'static/assets', to: 'mix' },
        { from: 'static/dashboard', to: 'dashboard' },
        { from: 'static/assets', to: 'dashboard' },
        { from: 'static/assets' },
      ],
    }),
    new MiniCssExtractPlugin({
      filename: '[name].css',
    }),
    new webpack.EnvironmentPlugin({
      VERSION: process.env.CARGO_PKG_VERSION || process.env.npm_package_version,
      WEBPACK_DEV_SERVER: process.env.WEBPACK_DEV_SERVER || '',
    }),
  ],
  devtool: is_prod ? false : 'source-map',
};

if (is_prod) {
  config.plugins = (config.plugins || []).concat([
    new MinifyHtmlWebpackPlugin({
      afterBuild: true,
      src: 'public',
      dest: 'public',
      ignoreFileNameRegex: /\.[^h.][^t.]?[^m.]?[^l.]?[^.]*$/,
      rules: {
        collapseBooleanAttributes: true,
        collapseWhitespace: true,
        removeAttributeQuotes: true,
        removeComments: true,
        minifyJS: true,
      },
    }),
  ]);
}

export default config;
