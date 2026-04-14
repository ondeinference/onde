const { getDefaultConfig } = require("expo/metro-config");
const path = require("path");

// The SDK root — one level up from this example app
const sdkRoot = path.resolve(__dirname, "..");

const config = getDefaultConfig(__dirname);

// Watch the SDK source so Metro picks up changes without a rebuild
config.watchFolders = [sdkRoot];

// Resolve @ondeinference/react-native to the SDK's TypeScript source
// directly, bypassing the compiled build/ output.
config.resolver.extraNodeModules = {
  "@ondeinference/react-native": sdkRoot,
};

// When Metro processes files from the SDK directory (outside this example app),
// it needs a fallback path to find transitive dependencies like @babel/runtime.
// Without this, Babel-transformed async/await in SDK files fails to resolve
// @babel/runtime/helpers/asyncToGenerator because the SDK's own node_modules
// is blocked below and the example's node_modules isn't in the resolution path.
config.resolver.nodeModulesPaths = [path.resolve(__dirname, "node_modules")];

// Block the SDK's own node_modules from Metro. Without this, Metro picks up
// react-native@0.85 (installed as a peer dep in sdk/react-native/node_modules/) instead
// of the example's own react-native@0.76 — causing syntax errors from newer RN
// internals that use pattern matching syntax not supported by this Babel config.
config.resolver.blockList = [
  new RegExp(
    path.resolve(sdkRoot, "node_modules").replace(/\\/g, "\\\\") + "/.*",
  ),
];

module.exports = config;
