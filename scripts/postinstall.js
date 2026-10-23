// Rebuilds native dependencies for Electron (electron-builder install-app-deps)
// without letting a failure abort a local `npm ci`.
//
// The only native dependency is cbor-extract, an optional accelerator for
// cbor-x: when it is missing, cbor-x falls back to pure JavaScript. Its source
// build can fail on some local toolchains (GCC rejects the V8 headers shipped
// with recent Electron), which used to stop the whole install for nothing.
//
// CI (any `CI` environment variable, set by GitHub Actions) and
// TFL_STRICT_POSTINSTALL=1 keep the old behaviour and fail, so packaging never
// silently ships without a rebuild it expected.
const { spawnSync } = require("node:child_process");

const strict = Boolean(process.env.CI) || process.env.TFL_STRICT_POSTINSTALL === "1";

// npm puts node_modules/.bin on PATH for lifecycle scripts; the argument list
// is fixed, so running through the shell (needed for .cmd shims on Windows)
// takes no outside input.
const result = spawnSync("electron-builder", ["install-app-deps"], {
  stdio: "inherit",
  shell: true,
});

if (result.status === 0) {
  process.exit(0);
}

if (strict) {
  process.exit(result.status ?? 1);
}

console.warn(`
[postinstall] Rebuilding native dependencies for Electron failed (see above).
[postinstall] Continuing: the only native module, cbor-extract, is optional and
[postinstall] cbor-x falls back to pure JavaScript without it. To build it, try
[postinstall]   CC=clang CXX=clang++ npm run postinstall
[postinstall] Set TFL_STRICT_POSTINSTALL=1 to make this failure fatal.
`);
