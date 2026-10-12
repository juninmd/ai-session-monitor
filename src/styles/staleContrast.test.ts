import { describe, expect, it } from 'vitest';
import shell from './shell.css?raw';
import tokens from './tokens.css?raw';

type Rgb = readonly [number, number, number];
type Theme = 'light' | 'dark';

/** A `--name: light-dark(#hex, #hex);` token, read from the stylesheet the app ships. */
function token(name: string): Record<Theme, Rgb> {
  const hex = '#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})';
  const found = new RegExp(`--${name}:\\s*light-dark\\(\\s*${hex}\\s*,\\s*${hex}\\s*\\)`, 'i').exec(
    tokens,
  );
  if (found === null) throw new Error(`--${name} is not a light-dark() pair of hex colours`);
  const channels = found.slice(1, 7).map((part) => Number.parseInt(part, 16));
  return {
    light: [channels[0] ?? 0, channels[1] ?? 0, channels[2] ?? 0],
    dark: [channels[3] ?? 0, channels[4] ?? 0, channels[5] ?? 0],
  };
}

const linear = (channel: number): number => {
  const c = channel / 255;
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
};
const luminance = ([r, g, b]: Rgb): number =>
  0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
const contrast = (a: Rgb, b: Rgb): number => {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return ((light ?? 0) + 0.05) / ((dark ?? 0) + 0.05);
};
/** What a group opacity does to a colour: it is blended into whatever sits behind the group. */
const faded = (colour: Rgb, behind: Rgb, alpha: number): Rgb => [
  colour[0] * alpha + behind[0] * (1 - alpha),
  colour[1] * alpha + behind[1] * (1 - alpha),
  colour[2] * alpha + behind[2] * (1 - alpha),
];

interface Rule {
  selectors: string[];
  body: string;
}

/** The rules of the shell that are scoped to the stale block. */
function staleRules(): Rule[] {
  const bare = shell.replace(/\/\*[\s\S]*?\*\//g, '');
  return [...bare.matchAll(/([^{}]+)\{([^{}]*)\}/g)]
    .map(([, selector = '', body = '']) => ({
      selectors: selector.split(',').map((part) => part.trim()),
      body,
    }))
    .filter((rule) => rule.selectors.some((selector) => selector.includes('[data-stale')));
}

const opacityOf = (rule: Rule): number | null => {
  const value = /(?:^|[;\s])opacity:\s*([\d.]+)/.exec(rule.body)?.[1];
  return value === undefined ? null : Number(value);
};

const BLOCK = '.stack[data-stale="true"]';
const AA = 4.5;

describe('stale numbers', () => {
  const surface = token('solid');
  const grounds = { card: token('raise'), flat: token('solid') };
  const inks = { 'ink-2': token('ink-2'), 'ink-3': token('ink-3') };

  it.each(['light', 'dark'] as const)(
    'keep secondary text at WCAG AA (4.5:1) in the %s theme once the block is dimmed',
    (theme) => {
      const block = staleRules().find((rule) => rule.selectors.includes(BLOCK));
      if (block === undefined) throw new Error('the stale rule moved: point this test at its home');
      const alpha = opacityOf(block) ?? 1;
      for (const [ground, background] of Object.entries(grounds)) {
        for (const [name, ink] of Object.entries(inks)) {
          const seen = contrast(
            faded(ink[theme], surface[theme], alpha),
            faded(background[theme], surface[theme], alpha),
          );
          const where = `${name} on ${ground}, ${theme}, group opacity ${alpha}`;
          expect(seen, where).toBeGreaterThanOrEqual(AA);
        }
      }
    },
  );

  it('fade only the bars: no opacity sits above anything that holds text', () => {
    // The gauge also holds the hover tooltip, so fading it whole drops that text under AA.
    const fading = staleRules().filter((rule) => opacityOf(rule) !== null);
    const aboveText = fading
      .flatMap((rule) => rule.selectors)
      .filter((selector) => !/\.gauge(?: meter|-void)$/.test(selector));
    expect(aboveText).toEqual([]);
  });
});
