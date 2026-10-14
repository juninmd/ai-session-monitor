// Checks that the versions pinned in Cargo.lock have soaked for >= SOAK_DAYS on crates.io.
// Usage: node scripts/soak-lock.mjs [--all] [--days 1]
//   default: direct dependencies of the workspace crates only; --all: every registry package.
// Exit code 1 when any checked version is younger than the soak window.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const args = process.argv.slice(2);
const all = args.includes('--all');
const daysIdx = args.indexOf('--days');
const SOAK_DAYS = daysIdx >= 0 ? Number(args[daysIdx + 1]) : 1;
const cutoff = Date.now() - SOAK_DAYS * 86_400_000;

const root = process.cwd();
const manifests = [join(root, 'Cargo.toml')];
for (const dir of ['crates', 'src-tauri']) {
  const base = join(root, dir);
  if (!existsSync(base)) continue;
  if (dir === 'src-tauri') manifests.push(join(base, 'Cargo.toml'));
  else for (const d of readdirSync(base)) manifests.push(join(base, d, 'Cargo.toml'));
}

// Names of third-party dependencies: lines in any "*dependencies*" table that are not path deps.
function directNames() {
  const names = new Set();
  for (const file of manifests) {
    if (!existsSync(file)) continue;
    let inDeps = false;
    for (const line of readFileSync(file, 'utf8').split(/\r?\n/)) {
      const header = line.match(/^\[(.+)\]\s*$/);
      if (header) {
        inDeps = /dependencies/.test(header[1]) && !header[1].includes('lints');
        continue;
      }
      if (!inDeps) continue;
      const m = line.match(/^([A-Za-z0-9_-]+)\s*(=|\.)/);
      if (!m || /\bpath\s*=/.test(line)) continue;
      names.add(m[1]);
    }
  }
  return names;
}

function lockedPackages() {
  const text = readFileSync(join(root, 'Cargo.lock'), 'utf8');
  const out = [];
  for (const block of text.split('[[package]]').slice(1)) {
    const name = block.match(/^name = "(.+)"$/m)?.[1];
    const version = block.match(/^version = "(.+)"$/m)?.[1];
    const source = block.match(/^source = "(.+)"$/m)?.[1] ?? '';
    if (name && version && source.startsWith('registry+')) out.push({ name, version });
  }
  return out;
}

const wanted = directNames();
const packages = lockedPackages().filter((p) => all || wanted.has(p.name));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const young = [];
let failed = 0;

for (const p of packages) {
  try {
    const res = await fetch(`https://crates.io/api/v1/crates/${p.name}/${p.version}`, {
      headers: { 'user-agent': 'ai-session-monitor-soak-lock (local dev script)' },
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const created = Date.parse((await res.json()).version.created_at);
    const ageDays = (Date.now() - created) / 86_400_000;
    const flag = created > cutoff ? '  <-- YOUNGER THAN SOAK' : '';
    if (flag) young.push(`${p.name} ${p.version}`);
    if (flag || !all)
      console.log(`${p.name.padEnd(30)} ${p.version.padEnd(12)} ${ageDays.toFixed(1)} d${flag}`);
  } catch (e) {
    failed++;
    console.log(`${p.name.padEnd(30)} ${p.version.padEnd(12)} ERROR ${e.message}`);
  }
  await sleep(1100);
}

console.log(
  `\nchecked ${packages.length} package(s) (${all ? 'all registry' : 'direct'}), ` +
    `${young.length} younger than ${SOAK_DAYS} d, ${failed} lookup error(s)`,
);
process.exit(young.length > 0 || failed > 0 ? 1 : 0);
