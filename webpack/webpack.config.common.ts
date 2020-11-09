import * as path from 'path';
import * as webpack from 'webpack';
import * as RawWasmPackPlugin from '@wasm-tool/wasm-pack-plugin';
import { CleanWebpackPlugin } from 'clean-webpack-plugin';
import TsconfigPathsPlugin from 'tsconfig-paths-webpack-plugin';
import * as HtmlEntryLoader from 'html-entry-loader';

const WasmPackPlugin = RawWasmPackPlugin as unknown as new (options: RawWasmPackPlugin.WasmPackPluginOptions) => webpack.WebpackPluginInstance;

const root = path.resolve(__dirname, '..');
const dist = path.resolve(root, 'client/dist');

const configuration: webpack.Configuration = {
  context: root,
  entry: {
    model_viewer: 'client/client.html',
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
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new HtmlEntryLoader.EntryExtractPlugin(),

    new WasmPackPlugin({
      crateDirectory: path.resolve(root, 'client'),
      outDir: path.resolve(root, 'client/pkg'),
      outName: 'index',
    }),
    new CleanWebpackPlugin(),
  ],
};

export default configuration;
