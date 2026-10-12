// Builds the aism-statusline-shim bin and stages it where Tauri's `externalBin` expects it:
//   src-tauri/binaries/statusline-shim-<host triple>[.exe]
// Usage: node src-tauri/scripts/prepare-sidecar.mjs [--release]   (default: debug build)
// `tauri build` runs it with --release through `beforeBuildCommand`; src-tauri/build.rs
// refuses a release build when the staged file is missing.
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, renameSync, rmSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const SHIM = 'statusline-shim';
// Cargo bin name differs from the sidecar stem so tauri-build's copy cannot overwrite it.
const BIN = 'aism-statusline-shim';
// A real executable is far larger; matches the guard in build.rs.
const MIN_BYTES = 4096;

const release = process.argv.includes('--release');
const exe = process.platform === 'win32' ? '.exe' : '';

// Argument arrays only: nothing here goes through a shell.
function capture(command, args) {
  return execFileSync(command, args, {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'inherit'],
  });
}

const hostLine = capture('rustc', ['-vV'])
  .split('\n')
  .find((line) => line.startsWith('host: '));
if (!hostLine) throw new Error('rustc -vV printed no host triple');
const triple = hostLine.slice('host: '.length).trim();

execFileSync('cargo', ['build', '--quiet', '-p', SHIM, ...(release ? ['--release'] : [])], {
  cwd: ROOT,
  stdio: 'inherit',
});

const { target_directory: targetDir } = JSON.parse(
  capture('cargo', ['metadata', '--no-deps', '--format-version', '1']),
);
const built = join(targetDir, release ? 'release' : 'debug', `${BIN}${exe}`);
if (statSync(built).size < MIN_BYTES) throw new Error(`${built} is too small to be the shim`);

const outDir = join(ROOT, 'src-tauri', 'binaries');
const staged = join(outDir, `${SHIM}-${triple}${exe}`);
const partial = `${staged}.partial`;
mkdirSync(outDir, { recursive: true });
try {
  copyFileSync(built, partial);
  renameSync(partial, staged);
} finally {
  rmSync(partial, { force: true });
}
process.stdout.write(`staged ${staged} (${statSync(staged).size} bytes)\n`);
