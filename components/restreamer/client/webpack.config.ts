import path from 'path';
import webpack from 'webpack';

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
    'full-stream/main': './src/AppFullStream.ts',
  },
  resolve: {
    alias: {
      svelte: path.resolve('node_modules', 'svelte'),
    },
    extensions: ['.mjs', '.js', '.ts', '.svelte'],
    mainFields: ['svelte', 'browser', 'module', 'main'],
  },
  output: {
    path: path.join(__dirname, '/public'),
    filename: '[name].js',
    chunkFilename: '[name].[id].js',
  },
  module: {
    rules: [
      {
        test: /\.svelte$/,
        use: {
          loader: 'svelte-loader',
          options: {
            preprocess: SveltePreprocess(),
            emitCss: true,
            hotReload: true,
            compilerOptions: {
              dev: !is_prod,
            }
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
          MiniCssExtractPlugin.loader,
          {
            loader: 'css-loader',
            options: {
              url: false,
            }
          },
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
        { from: 'static/full-stream', to: 'full-stream' },
        { from: 'static/assets', to: 'full-stream' },
        { from: 'static/assets' },
      ],
    }),
    new MiniCssExtractPlugin({
      filename: is_prod ? '[name].[contenthash].css' : '[name].css',
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
