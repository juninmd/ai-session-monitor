// The names of the dev scenarios (`?mock=<name>`), apart from the fabricated data they select: the
// palette lists them, and a production bundle must not carry the fixtures along with the list.
export const SCENARIO_NAMES = [
  'demo',
  'fresh',
  'healthy',
  'critical',
  'stale',
  'errors',
  'loading',
] as const;
export type ScenarioName = (typeof SCENARIO_NAMES)[number];

/** Anything that is not a known name falls back to the demo: the query string is untrusted. */
export function scenarioFromSearch(search: string): ScenarioName {
  const raw = new URLSearchParams(search).get('mock');
  return SCENARIO_NAMES.find((name) => name === raw) ?? 'demo';
}
