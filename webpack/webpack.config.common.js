const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');
const HtmlEntryLoader = require('html-entry-loader');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const GrpcWebPlugin = require('grpc-webpack-plugin');

const root = path.resolve(__dirname, '..');
const dist = path.resolve(root, 'client/dist');

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

    new GrpcWebPlugin({
      protoPath: path.resolve(root, 'proto'),
      protoFiles: ['bouncer.proto'],
      outputType: 'grpc-web',
      importStyle: 'typescript',
      binary: true,
      outDir: path.resolve('client/src/proto'),
      extra: [`--js_out=import_style=commonjs,binary:${path.resolve('client/src/proto')}`],
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(root, 'client/wasm'),
      outDir: path.resolve(root, 'client/wasm/pkg'),
      outName: 'index',
    }),
    new CleanWebpackPlugin(),
  ],
};
