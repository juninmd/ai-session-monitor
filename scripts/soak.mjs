// Resolve the newest stable version of each package that has soaked for >= SOAK_DAYS.
// Usage: node scripts/soak.mjs npm react,vite [days]  |  node scripts/soak.mjs crates tauri,keyring [days]
const [, , registry, list, soakArg] = process.argv;
const SOAK_DAYS = Number(soakArg ?? 1);
const usable =
  ['npm', 'crates'].includes(registry) && list && Number.isFinite(SOAK_DAYS) && SOAK_DAYS >= 0;
if (!usable) {
  console.error('Usage: node scripts/soak.mjs <npm|crates> <name,name,...> [days]');
  process.exit(2);
}
const now = Date.now();
const cutoff = now - SOAK_DAYS * 86_400_000;
const names = list.split(',').filter(Boolean);

const stable = (v) => /^\d+\.\d+\.\d+$/.test(v);
const cmp = (a, b) => {
  const pa = a.split('.').map(Number);
  const pb = b.split('.').map(Number);
  for (let i = 0; i < 3; i++) if (pa[i] !== pb[i]) return pa[i] - pb[i];
  return 0;
};

async function npm(name) {
  const res = await fetch(`https://registry.npmjs.org/${name.replace('/', '%2F')}`, {
    headers: { accept: 'application/json' },
  });
  if (!res.ok) throw new Error(`${name}: HTTP ${res.status}`);
  const doc = await res.json();
  const latest = doc['dist-tags']?.latest;
  const rows = Object.entries(doc.time ?? {})
    .filter(([v]) => stable(v) && !doc.versions?.[v]?.deprecated)
    .map(([v, t]) => ({ v, t: Date.parse(t) }));
  const soaked = rows.filter((r) => r.t <= cutoff).sort((a, b) => cmp(a.v, b.v));
  const pick = soaked.at(-1);
  const latestRow = rows.find((r) => r.v === latest);
  return {
    name,
    latest,
    latestDate: latestRow && new Date(latestRow.t).toISOString().slice(0, 10),
    pick: pick?.v,
    pickDate: pick && new Date(pick.t).toISOString().slice(0, 10),
  };
}

async function crates(name) {
  const res = await fetch(`https://crates.io/api/v1/crates/${name}/versions`, {
    headers: {
      'user-agent': 'ai-session-monitor-soak-check (local dev script)',
      accept: 'application/json',
    },
  });
  if (!res.ok) throw new Error(`${name}: HTTP ${res.status}`);
  const doc = await res.json();
  const rows = doc.versions
    .filter((v) => !v.yanked && stable(v.num))
    .map((v) => ({ v: v.num, t: Date.parse(v.created_at), rv: v.rust_version }));
  const newest = [...rows].sort((a, b) => cmp(a.v, b.v)).at(-1);
  const soaked = rows.filter((r) => r.t <= cutoff).sort((a, b) => cmp(a.v, b.v));
  const pick = soaked.at(-1);
  return {
    name,
    latest: newest?.v,
    latestDate: newest && new Date(newest.t).toISOString().slice(0, 10),
    pick: pick?.v,
    pickDate: pick && new Date(pick.t).toISOString().slice(0, 10),
    msrv: pick?.rv ?? null,
  };
}

const fn = registry === 'npm' ? npm : crates;
const out = [];
for (const name of names) {
  try {
    out.push(await fn(name));
  } catch (e) {
    out.push({ name, error: String(e.message ?? e) });
  }
}
for (const r of out) {
  if (r.error) console.log(`${r.name.padEnd(34)} ERROR ${r.error}`);
  else {
    const flag = r.pick === r.latest ? '' : '  <-- newer than soak';
    console.log(
      `${r.name.padEnd(34)} pick ${String(r.pick).padEnd(9)} (${r.pickDate})  latest ${r.latest} (${r.latestDate})${r.msrv ? `  msrv ${r.msrv}` : ''}${flag}`,
    );
  }
}
