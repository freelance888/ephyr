import path from 'path';
import webpack from 'webpack';
import 'webpack-dev-server';

import CopyPlugin from 'copy-webpack-plugin';
import MiniCssExtractPlugin from 'mini-css-extract-plugin';
import MinifyHtmlWebpackPlugin from 'minify-html-webpack-plugin';
import SveltePreprocess from 'svelte-preprocess';
import HtmlWebpackPlugin from 'html-webpack-plugin';

const is_prod = process.env.NODE_ENV === 'production';
const mode = is_prod ? 'production' : 'development';
const isDevServer = process.argv.some((v) => v.includes('webpack-dev-server'));
const EPHYR_DEV_HOST_PORT = setEphyrDevHostPort(isDevServer);

function setEphyrDevHostPort(isDevServer: boolean): string | null {
  if (isDevServer) {
    let port = process.env.EPHYR_RESTREAMER_CLIENT_HTTP_PORT;
    if (port === undefined) {
      console.warn(
        'No `EPHYR_RESTREAMER_CLIENT_HTTP_PORT` env var set, use default `:80`'
      );
      port = '80';
    }
    console.log(
      `Use host from browser and port \`${port}\` as backend server address`
    );
    return port;
  } else {
    console.log('Use host and port from browser as backend server address');
    return null;
  }
}

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
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      chunks: 'all',
    },
  },
  devServer: {
    static: path.join(__dirname, 'public'),
    compress: true,
    port: 8080,
    host: '0.0.0.0',
    client: {
      webSocketURL: isDevServer
        ? `ws://0.0.0.0:${EPHYR_DEV_HOST_PORT}/api`
        : undefined,
    },
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
            preprocess: SveltePreprocess(),
            emitCss: true,
            hotReload: true,
            compilerOptions: {
              dev: !is_prod,
            },
            onwarn: (warning, handler) => {
              if (warning.code.startsWith('a11y')) return;

              // Handle all other warnings normally
              handler(warning);
            },
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
            },
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
        { from: 'static/assets' },
        { from: 'static/assets', to: 'mix' },
        { from: 'static/assets', to: 'dashboard' },
        { from: 'static/assets', to: 'full-stream' },
      ],
    }),
    new MiniCssExtractPlugin({
      filename: is_prod ? '[name].[contenthash].css' : '[name].css',
      ignoreOrder: true,
    }),
    new HtmlWebpackPlugin({
      title: 'Ephyr re-streamer',
      filename: 'index.html',
      template: 'static/index.html',
      baseHref: '/',
      chunks: ['main'],
    }),
    new HtmlWebpackPlugin({
      title: 'Ephyr Mixin',
      filename: 'mix/index.html',
      template: 'static/index.html',
      baseHref: '/mix',
      chunks: ['mix/main'],
    }),
    new HtmlWebpackPlugin({
      title: 'Ephyr Dashboard',
      filename: 'dashboard/index.html',
      template: 'static/index.html',
      baseHref: '/dashboard',
      chunks: ['dashboard/main'],
    }),
    new HtmlWebpackPlugin({
      title: 'Ephyr Full Stream',
      filename: 'full-stream/index.html',
      template: 'static/index.html',
      baseHref: '/full-stream',
      chunks: ['full-stream/main'],
    }),
    new webpack.EnvironmentPlugin({
      VERSION: process.env.CARGO_PKG_VERSION || process.env.npm_package_version,
      WEBPACK_DEV_SERVER: process.env.WEBPACK_DEV_SERVER || '',
      EPHYR_DEV_HOST_PORT: EPHYR_DEV_HOST_PORT,
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
