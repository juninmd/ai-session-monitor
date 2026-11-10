// Fabricated data stays out of the shipped bundle: only the mock entry is loaded, on demand, and
// only outside Tauri. A static import of a fixture from production code would ship it to everyone.
import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>('../**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
});

/** The scenario names are a plain list; the palette may read it. Types cost nothing at runtime. */
const ALLOWED = /from '[./]+\/mock\/scenarioNames'/;
const MOCK_IMPORT = /^import (?!type\b)[^;]*from '[./]*\/?mock(\/[\w]+)?';/gm;
const production = (path: string) =>
  // `./` is this folder: the test helpers, which build fabricated data on purpose.
  !path.startsWith('./') &&
  !path.includes('/mock/') &&
  !/\.test\.tsx?$/.test(path) &&
  !/[tT]estHarness|settingsHarness/.test(path);

describe('module boundaries', () => {
  it('finds the sources it is meant to check', () => {
    expect(Object.keys(sources).filter(production).length).toBeGreaterThan(150);
  });

  it('keeps fabricated fixtures out of production modules', () => {
    const leaks = Object.entries(sources)
      .filter(([path]) => production(path))
      .flatMap(([path, code]) =>
        (code.match(MOCK_IMPORT) ?? [])
          .filter((line) => !ALLOWED.test(line))
          .map((line) => `${path}: ${line}`),
      );
    expect(leaks).toEqual([]);
  });

  it('would catch the import it guards against', () => {
    expect("import { SCENARIO_NAMES } from '../mock/scenarios';".match(MOCK_IMPORT)).not.toBeNull();
    expect("import type { MockBackend } from '../mock';".match(MOCK_IMPORT)).toBeNull();
  });
});
