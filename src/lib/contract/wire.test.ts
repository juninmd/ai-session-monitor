// The golden samples only exercise some variants of each wire enum, so a rename on the others would
// pass every other test and fail on a user's machine, when the strict parse drops the snapshot.
// This reads the Rust enum definitions themselves and compares their wire names with the Zod lists.
import { describe, expect, it } from 'vitest';
import settingsRs from '../../../crates/usage-model/src/settings.rs?raw';
import updateRs from '../../../crates/usage-model/src/update.rs?raw';
import viewRs from '../../../crates/usage-model/src/view.rs?raw';
import { LITELLM_SCOPES, TimeFormatSchema, WeekFormatSchema } from './settings';
import { UPDATE_FAILURES, UPDATE_PHASES } from './update';
import { AMOUNT_UNITS, ERROR_CODES, SEVERITIES, WINDOW_KINDS } from './view';

/** serde's `rename_all = "snake_case"` for a variant name: `DaysHours` -> `days_hours`. */
const snakeCase = (variant: string): string => variant.replace(/\B([A-Z])/g, '_$1').toLowerCase();

/**
 * The wire names of `pub enum <name>`, which must be a plain list of unit variants under
 * `rename_all = "snake_case"`. Anything else throws, so this never reads a stale shape in silence.
 */
function wireNames(source: string, name: string): string[] {
  const text = source.replace(/\r\n/g, '\n');
  const header = `#\\[serde\\(rename_all = "snake_case"\\)\\]\\npub enum ${name} \\{\\n`;
  const found = new RegExp(`${header}([\\s\\S]*?)\\n\\}`).exec(text);
  if (found === null) throw new Error(`${name}: not a snake_case enum in the Rust source`);
  const lines = (found[1] ?? '').split('\n').map((line) => line.trim());
  return lines
    .filter((line) => line !== '' && !line.startsWith('//'))
    .map((line) => {
      const variant = /^([A-Z][A-Za-z0-9]*),$/.exec(line)?.[1];
      if (variant === undefined) throw new Error(`${name}: cannot read the variant "${line}"`);
      return snakeCase(variant);
    });
}

describe('wire enums', () => {
  it.each([
    ['WindowKind', viewRs, WINDOW_KINDS],
    ['UpdatePhase', updateRs, UPDATE_PHASES],
    ['UpdateFailure', updateRs, UPDATE_FAILURES],
    ['AmountUnit', viewRs, AMOUNT_UNITS],
    ['Severity', viewRs, SEVERITIES],
    ['ErrorCode', viewRs, ERROR_CODES],
    ['TimeFormat', settingsRs, TimeFormatSchema.options],
    ['WeekFormat', settingsRs, WeekFormatSchema.options],
    ['LitellmScope', settingsRs, LITELLM_SCOPES],
  ] as const)('%s lists exactly the wire names of the Rust enum', (name, source, mirror) => {
    expect([...mirror].sort()).toEqual(wireNames(source, name).sort());
  });
});

describe('wireNames', () => {
  const probe = (body: string) =>
    `#[serde(rename_all = "snake_case")]\npub enum Probe {\n${body}\n}`;

  it('skips docs and reads CRLF checkouts the same as LF ones', () => {
    const source = probe('    /// Doc.\n    FiveHour,\n\n    Usd,');
    expect(wireNames(source, 'Probe')).toEqual(['five_hour', 'usd']);
    expect(wireNames(source.replace(/\n/g, '\r\n'), 'Probe')).toEqual(['five_hour', 'usd']);
  });

  it('throws on a shape it cannot read instead of returning a shorter list', () => {
    expect(() => wireNames(probe('    #[serde(rename = "x")]\n    One,'), 'Probe')).toThrow(
      'cannot read the variant',
    );
    expect(() => wireNames(probe('    One(String),'), 'Probe')).toThrow('cannot read the variant');
    expect(() => wireNames('pub enum Probe {\n    One,\n}', 'Probe')).toThrow('not a snake_case');
  });
});
