const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');
const HtmlEntryLoader = require('html-entry-loader');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const WebpackShellPluginNext = require('webpack-shell-plugin-next');

const root = path.resolve(__dirname, '..');
const dist = path.resolve(root, 'client/dist');

const protos = ['bouncer.proto'];
const grpcWebBuild =
  `node node_modules/protoc-gen-grpc/bin/protoc-gen-grpc.js -I=${path.resolve(root, 'node_modules/protoc/protoc/include')} -I=${path.resolve(root, 'proto')} ${protos.join(' ')}` +
  ` --js_out=import_style=commonjs,binary:${path.resolve('client/src/proto')}` +
  ` --grpc-web_out=import_style=commonjs+dts,mode=grpcweb:${path.resolve('client/src/proto')}`;

module.exports = {
  context: root,
  entry: {
    client: 'client/src/client.html',
  },
  experiments: {
    asyncWebAssembly: true,
  },
  output: {
    path: dist,
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.(html)$/,
        use: [
          {
            loader: 'html-entry-loader',
            options: {
              minimize: true,
            },
          },
        ],
      },
      {
        test: /\.ts$/,
        use: [
          {
            loader: 'ts-loader',
            options: {
              onlyCompileBundledFiles: true,
              compilerOptions: {
                module: 'esnext',
              },
            },
          },
        ],
      },
    ],
  },
  resolve: {
    extensions: ['.ts', '.js'],
    plugins: [new TsconfigPathsPlugin()],
    alias: {
      proto: path.resolve(root, 'client/src/proto'),
    },
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new HtmlEntryLoader.EntryExtractPlugin(),
    new WebpackShellPluginNext({
      onBeforeNormalRun: {
        scripts: [grpcWebBuild],
      },
      onWatchRun: {
        scripts: [grpcWebBuild],
      },
      onBeforeBuild: {
        scripts: [grpcWebBuild],
      },
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(root, 'client/wasm'),
      outDir: path.resolve(root, 'client/wasm/pkg'),
      outName: 'index',
    }),
    new CleanWebpackPlugin(),
  ],
};
