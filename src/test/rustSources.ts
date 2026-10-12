// The Rust crates' production code as text, for tests that pin the UI to what the collectors
// really send. Only string literals are matched: nothing here runs Rust or opens a profile.
const sources = import.meta.glob<string>('../../crates/*/src/**/*.rs', {
  query: '?raw',
  import: 'default',
  eager: true,
});

// Test-only files and inline test modules emit nothing a person can see.
const TEST_FILE = /(?:^|\/)(?:tests?|fixtures?)\.rs$|\/tests\/|\/fake[^/]*\.rs$/;
const INLINE_TESTS = /^#\[cfg\(test\)\]\s*\n\s*mod\s+\w+/m;

export const isTestFile = (path: string): boolean => TEST_FILE.test(path);

/** A file's code before its inline `#[cfg(test)] mod`. */
export const productionCode = (source: string): string => source.split(INLINE_TESTS)[0] ?? '';

const crates = new Map<string, string>();
for (const [path, text] of Object.entries(sources)) {
  const name = /crates\/([^/]+)\/src\//.exec(path)?.[1];
  if (name === undefined || isTestFile(path)) continue;
  crates.set(name, `${crates.get(name) ?? ''}\n${productionCode(text)}`);
}

/** The production code of one crate (`collector-claude`, `app-core`, ...); empty if it is unknown. */
export const rustOf = (crate: string): string => crates.get(crate) ?? '';

export const COLLECTORS = [
  'collector-antigravity',
  'collector-claude',
  'collector-codex',
  'collector-deepseek',
  'collector-deepinfra',
  'collector-moonshot',
  'collector-novita',
  'collector-ollama',
  'collector-minimax',
  'collector-kimi',
  'collector-litellm',
  'collector-opencode',
  'collector-openrouter',
  'collector-zai',
] as const;

/** Everything a person can be shown comes from a collector or from the core that validates input. */
export const emitters = (): string => [...COLLECTORS, 'app-core'].map(rustOf).join('\n');

/** Every capture group of every match, de-duplicated and sorted. */
export function scan(source: string, pattern: RegExp): string[] {
  const found = [...source.matchAll(pattern)].flatMap((match) =>
    match.slice(1).filter((group): group is string => group !== undefined),
  );
  return [...new Set(found)].sort();
}

const FACT_STRUCT = String.raw`Fact\s*\{\s*id:\s*"([a-z][a-z0-9_]*)"`;
const FACT_ROW = String.raw`\(\s*"([a-z][a-z0-9_]*)"\s*,\s*[\w.]+(?:\s+as\s+f64)?\s*,\s*(?:"[a-z_]+"|[A-Z_]+)\s*\)`;
const FIELD_CALL = String.raw`\b(?:invalid_input|invalid_url|invalid|check_path|optional_path)\(\s*"([A-Za-z]+)"`;
const FIELD_PARAM = String.raw`with_param\(\s*"field"\s*,\s*"([A-Za-z]+)"\s*\)`;

/** The shapes in which the crates spell what the UI has to word; emitted.test.ts pins each one. */
export const PATTERN = {
  noticeCode:
    /const\s+[A-Z_]+:\s*&str\s*=\s*"((?:claude|codex|antigravity|litellm|opencode)_[a-z_]+)"/g,
  fact: new RegExp(`${FACT_STRUCT}|${FACT_ROW}`, 'g'),
  installReason: /\bfailed\(\s*"([a-z_]+)"\s*,/g,
  tlsReason: /"(ca_[a-z_]+)"/g,
  endpointReason: /\bchanged\(\s*"([a-z_]+)"\s*\)/g,
  reasonParam: /with_param\(\s*"reason"\s*,\s*"([a-z_]+)"\s*\)/g,
  field: new RegExp(`${FIELD_CALL}|${FIELD_PARAM}`, 'g'),
} as const;
