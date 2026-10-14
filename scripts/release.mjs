import { appendFileSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { readCommits, run } from './release-git.mjs';
import { verifyReleaseFiles } from './verify-release.mjs';

export { readCommits, run, workflowsTree } from './release-git.mjs';

export function commitTitle(message) {
  const lines = message
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  return lines[0]?.startsWith('Merge pull request') ? (lines[1] ?? lines[0]) : (lines[0] ?? '');
}

export function releaseBump(message) {
  const title = commitTitle(message);
  if (/^[a-z][\w-]*(?:\([^\r\n()]+\))?!:/i.test(title) || /^BREAKING[ -]CHANGE\s*:/im.test(message))
    return 'major';
  const type = /^([a-z][\w-]*)(?:\([^\r\n()]+\))?:/i.exec(title)?.[1]?.toLowerCase();
  if (type === 'feat') return 'minor';
  if (type === 'fix') return 'patch';
  return undefined;
}

export function bumpVersion(version, bump) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match) throw new Error(`Versão inválida: ${version}`);
  const [, majorText, minorText, patchText] = match;
  const [major, minor, patch] = [Number(majorText), Number(minorText), Number(patchText)];
  if (bump === 'major') return `${major + 1}.0.0`;
  if (bump === 'minor') return `${major}.${minor + 1}.0`;
  return `${major}.${minor}.${patch + 1}`;
}

const BUMP_RANK = { patch: 1, minor: 2, major: 3 };

function strongestBump(left, right) {
  if (!left || !right) return left ?? right;
  return BUMP_RANK[right] > BUMP_RANK[left] ? right : left;
}

function compareVersions(left, right) {
  const leftParts = left.split('.').map(Number);
  const rightParts = right.split('.').map(Number);
  for (let index = 0; index < 3; index += 1) {
    const difference = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (difference) return difference;
  }
  return 0;
}

const updaterBundles = {
  'windows-x86_64': /_x64-setup\.nsis\.zip$|_x64-setup\.exe$/,
  'windows-x86_64-nsis': /_x64-setup\.nsis\.zip$|_x64-setup\.exe$/,
};

function bundleName(assets, version, pattern) {
  return assets.find(
    (name) => name.includes(version) && pattern.test(name) && assets.includes(`${name}.sig`),
  );
}

export function hasRequiredAssets(assets, version) {
  return (
    assets.includes('latest.json') &&
    Object.values(updaterBundles).every((pattern) => bundleName(assets, version, pattern))
  );
}

export function planReleases(commits, tags, releases, previousTag) {
  let version = previousTag.replace(/^v/, '');
  let priorTag = previousTag;
  const candidates = [];
  let pending;
  for (const commit of commits) {
    if (!/^[0-9a-f]{40}$/.test(commit.sha)) throw new Error(`Commit inválido: ${commit.sha}`);
    pending = strongestBump(pending, releaseBump(commit.message));
    if (!pending || commit.taggable === false) continue;
    version = bumpVersion(version, pending);
    pending = undefined;
    const tag = `v${version}`;
    const taggedSha = tags.get(tag);
    if (taggedSha && taggedSha !== commit.sha)
      throw new Error(`${tag} já aponta para outro commit`);
    const release = releases.get(tag);
    if (release && !release.draft) {
      if (!hasRequiredAssets(release.assets, version))
        throw new Error(`${tag} foi publicada sem os instaladores completos`);
    } else {
      candidates.push({ sha: commit.sha, tag, version, previousTag: priorTag });
    }
    priorTag = tag;
  }
  return candidates;
}

function plan() {
  const repository =
    process.env.GITHUB_REPOSITORY ??
    run('gh', ['repo', 'view', '--json', 'nameWithOwner', '--jq', '.nameWithOwner']);
  const tagNames = run('git', ['tag', '--list', 'v[0-9]*.[0-9]*.[0-9]*']);
  const tags = new Map(
    (tagNames ? tagNames.split('\n') : []).map((tag) => [
      tag,
      run('git', ['rev-parse', `${tag}^{commit}`]),
    ]),
  );
  let releases = new Map();
  try {
    const releaseRows = run('gh', [
      'api',
      '--paginate',
      `repos/${repository}/releases?per_page=100`,
      '--jq',
      '.[] | {tag: .tag_name, draft: .draft, assets: [.assets[].name]} | @json',
    ]);
    releases = new Map(
      (releaseRows ? releaseRows.split('\n') : []).map((line) => {
        const row = JSON.parse(line);
        return [row.tag, { draft: row.draft, assets: row.assets }];
      }),
    );
  } catch {
    // If no releases yet exist
  }

  const publishedTags = [...releases.entries()]
    .filter(([tag, release]) => /^v\d+\.\d+\.\d+$/.test(tag) && !release.draft && tags.has(tag))
    .sort(([left], [right]) => compareVersions(right.slice(1), left.slice(1)));

  let previousTag = publishedTags.find(([tag]) => {
    try {
      run('git', ['merge-base', '--is-ancestor', tag, 'origin/master']);
      return true;
    } catch {
      return false;
    }
  })?.[0];

  let history;
  if (!previousTag) {
    previousTag = 'v0.0.0';
    const rows = run('git', ['rev-list', '--first-parent', '--reverse', 'origin/master']);
    history = rows ? rows.split('\n') : [];
  } else {
    const rows = run('git', [
      'rev-list',
      '--first-parent',
      '--reverse',
      `${previousTag}..origin/master`,
    ]);
    history = rows ? rows.split('\n') : [];
  }

  const commits = readCommits(history, 'origin/master');
  const pending = planReleases(commits, tags, releases, previousTag);
  if (process.env.GITHUB_OUTPUT) {
    appendFileSync(process.env.GITHUB_OUTPUT, `candidate=${JSON.stringify(pending[0] ?? {})}\n`);
    appendFileSync(process.env.GITHUB_OUTPUT, `more=${pending.length > 1}\n`);
  }
  console.log(JSON.stringify(pending));
}

const GROUPS = [
  {
    key: 'breaking',
    heading: '⚠️ Mudanças incompatíveis',
    one: 'mudança incompatível',
    many: 'mudanças incompatíveis',
  },
  { key: 'feat', heading: '✨ Novidades', one: 'novidade', many: 'novidades' },
  { key: 'fix', heading: '🐛 Correções', one: 'correção', many: 'correções' },
  {
    key: 'perf',
    heading: '⚡ Desempenho',
    one: 'ganho de desempenho',
    many: 'ganhos de desempenho',
  },
  {
    key: 'refactor',
    heading: '♻️ Melhorias internas',
    one: 'melhoria interna',
    many: 'melhorias internas',
  },
];

export function parseEntry(line, sha) {
  const match = /^([a-z][\w-]*)(?:\(([^)\r\n]+)\))?(!)?:\s+(.+)$/i.exec(line.trim());
  if (!match) return undefined;
  const subject = match[4].replace(/\s*\(#\d+\)\s*$/, '').trim();
  if (!subject) return undefined;
  return {
    type: match[1].toLowerCase(),
    scope: match[2]?.trim() ?? '',
    subject,
    breaking: Boolean(match[3]),
    sha,
  };
}

export function releaseNotes(message, candidate, repository) {
  const title = commitTitle(message);
  const root = `https://github.com/${repository}`;
  const entry = parseEntry(title, candidate.sha) ?? {
    type: 'feat',
    scope: '',
    subject: title,
    breaking: false,
    sha: candidate.sha,
  };
  const grouped = new Map();
  const key = entry.breaking ? 'breaking' : entry.type;
  grouped.set(key, [entry]);

  const sections = GROUPS.flatMap((g) => {
    const list = grouped.get(g.key);
    if (!list) return [];
    return [
      `## ${g.heading}\n\n${list.map((e) => `- ${e.subject} ([\`${e.sha.slice(0, 7)}\`](${root}/commit/${e.sha}))`).join('\n')}`,
    ];
  });

  const downloadTable = [
    '| Sistema | Instalador |',
    '|---|---|',
    `| 🪟 Windows | [Instalador (.exe)](${root}/releases/download/${candidate.tag}/AI.Session.Monitor_${candidate.version}_x64-setup.exe) |`,
  ].join('\n');

  return `${[
    `## AI Session Monitor ${candidate.tag}`,
    ...sections,
    `## ⬇️ Baixar\n\n${downloadTable}\n\nInstaladores assinados criptograficamente com Minisign. O manifesto e as assinaturas dos instaladores baixados são verificados antes da publicação.`,
  ].join('\n\n')}\n`;
}

function notes() {
  const [tag, sha, previousTag, path] = process.argv.slice(3);
  if (!tag || !sha || !previousTag || !path || !/^v\d+\.\d+\.\d+$/.test(tag))
    throw new Error('Argumentos inválidos');
  const repository =
    process.env.GITHUB_REPOSITORY ??
    run('gh', ['repo', 'view', '--json', 'nameWithOwner', '--jq', '.nameWithOwner']);
  const message = run('git', ['log', '-1', '--format=%B', sha]);
  writeFileSync(
    path,
    releaseNotes(message, { sha, tag, version: tag.slice(1), previousTag }, repository),
  );
}

function manifest() {
  const [releasePath, signaturesPath, notesPath, outputPath, tag, version] = process.argv.slice(3);
  if (!releasePath || !signaturesPath || !notesPath || !outputPath || !tag || !version)
    throw new Error('Argumentos inválidos');
  const release = JSON.parse(readFileSync(releasePath, 'utf8'));
  const signatures = new Map(
    readdirSync(signaturesPath)
      .filter((name) => name.endsWith('.sig'))
      .map((name) => [name, readFileSync(join(signaturesPath, name), 'utf8')]),
  );
  const repository =
    process.env.GITHUB_REPOSITORY ??
    run('gh', ['repo', 'view', '--json', 'nameWithOwner', '--jq', '.nameWithOwner']);

  const names = release.assets.map(({ name }) => name);
  const prefix = `https://github.com/${repository}/releases/download/${tag}/`;
  const platforms = Object.fromEntries(
    Object.entries(updaterBundles).map(([platform, pattern]) => {
      const name = bundleName(names, version, pattern);
      const asset = release.assets.find((item) => item.name === name);
      const signature = name ? signatures.get(`${name}.sig`)?.trim() : undefined;
      if (!asset || !signature)
        throw new Error(`Instalador ou assinatura ausente para ${platform}`);
      return [platform, { signature, url: `${prefix}${encodeURIComponent(asset.name)}` }];
    }),
  );

  const result = {
    version,
    notes: readFileSync(notesPath, 'utf8'),
    pub_date: new Date().toISOString(),
    platforms,
  };
  writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`);
}

if (process.argv[2] === 'plan') plan();
else if (process.argv[2] === 'notes') notes();
else if (process.argv[2] === 'manifest') manifest();
else if (process.argv[2] === 'verify') verifyReleaseFiles(process.argv.slice(3));
