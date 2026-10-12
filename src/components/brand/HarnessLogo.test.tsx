import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HARNESS_IDS, type HarnessId } from '../../lib/contract';
import { fingerprint, parseSvg, unsafeFindings } from '../../test/svg';
import { HarnessLogo } from './HarnessLogo';
import kimiFile from './original/kimi.mono.svg?raw';
import zaiFile from './original/zai.mono.svg?raw';
import antigravityFile from './vendor/antigravity.svg?raw';
import claudeFile from './vendor/claude.svg?raw';
import codexFile from './vendor/codex.svg?raw';
import litellmFile from './vendor/litellm.mono.svg?raw';
import opencodeFile from './vendor/opencode.mono.svg?raw';

/** The file each component was written from: sanitized upstream art, or (kimi, zai) an original glyph. */
const VENDORED: Record<HarnessId, string> = {
  claude: claudeFile,
  codex: codexFile,
  antigravity: antigravityFile,
  opencode: opencodeFile,
  kimi: kimiFile,
  zai: zaiFile,
  litellm: litellmFile,
};

const TONE: Record<HarnessId, 'color' | 'mono'> = {
  claude: 'color',
  codex: 'color',
  antigravity: 'color',
  opencode: 'mono',
  kimi: 'mono',
  zai: 'mono',
  litellm: 'mono',
};

/** Ids a logo defines for its own gradient, mask and filters. */
const DEFINED_IDS: Record<HarnessId, number> = {
  claude: 0,
  codex: 1,
  antigravity: 12,
  opencode: 0,
  kimi: 0,
  zai: 0,
  litellm: 0,
};

function logoOf(id: HarnessId, size?: number): SVGSVGElement {
  const { container } = render(<HarnessLogo id={id} size={size} />);
  const svgs = container.querySelectorAll('svg');
  expect(svgs).toHaveLength(1);
  return svgs[0] as SVGSVGElement;
}

describe('HarnessLogo', () => {
  it.each(HARNESS_IDS)('draws exactly one decorative svg for %s', (id) => {
    const svg = logoOf(id);
    expect(svg).toHaveAttribute('aria-hidden', 'true');
    expect(svg).toHaveAttribute('focusable', 'false');
    expect(svg).toHaveAttribute('data-logo', id);
    expect(svg).toHaveAttribute('viewBox');
    // The tab or row already names the tool; a title would add a second name and a tooltip.
    expect(svg.querySelector('title')).toBeNull();
  });

  it('gives every tool its own mark', () => {
    const marks = HARNESS_IDS.map((id) => {
      const { container, unmount } = render(<HarnessLogo id={id} />);
      const mark = fingerprint(container.querySelector('svg') as Element).join('\n');
      unmount();
      return mark;
    });
    expect(new Set(marks).size).toBe(HARNESS_IDS.length);
  });

  it('sizes the square box and passes a class through', () => {
    const { container } = render(<HarnessLogo id="codex" size={32} className="t-icon" />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '32');
    expect(svg).toHaveAttribute('height', '32');
    expect(svg).toHaveClass('logo', 't-icon');
  });

  it('defaults to a 20 px box', () => {
    expect(logoOf('claude')).toHaveAttribute('width', '20');
  });
});

describe('the vendored artwork', () => {
  it.each(HARNESS_IDS)('keeps the %s path data and paint exactly as in its vendored file', (id) => {
    expect(fingerprint(logoOf(id))).toEqual(fingerprint(parseSvg(VENDORED[id])));
  });

  it.each(HARNESS_IDS)('is one clean svg with a viewBox in the %s file', (id) => {
    const root = parseSvg(VENDORED[id]);
    expect(unsafeFindings(root)).toEqual([]);
    // jest-dom cannot inspect nodes of a parsed document, so ask the DOM directly.
    expect(root.hasAttribute('viewBox')).toBe(true);
    expect(root.querySelectorAll('svg')).toHaveLength(0);
  });

  it.each(HARNESS_IDS)('renders only plain vector markup for %s', (id) => {
    expect(unsafeFindings(logoOf(id))).toEqual([]);
  });

  it.each([
    ['a script', '<script>alert(1)</script>'],
    ['a style element', '<style>path{fill:red}</style>'],
    ['a foreignObject', '<foreignObject><div/></foreignObject>'],
    ['an image', '<image href="logo.png"/>'],
    ['an event handler', '<path d="M0 0" onload="alert(1)"/>'],
    ['an external href', '<use href="https://example.com/a.svg#b"/>'],
    ['a remote url()', '<path d="M0 0" fill="url(https://example.com/g)"/>'],
    ['an inline style', '<path d="M0 0" style="fill:red"/>'],
  ])('catches %s, so the audit is not vacuous', (_, markup) => {
    const root = parseSvg(
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1">${markup}</svg>`,
    );
    expect(unsafeFindings(root)).not.toEqual([]);
  });
});

describe('gradient, mask and filter ids', () => {
  it.each(HARNESS_IDS)('stay unique when %s is on screen twice', (id) => {
    const { container } = render(
      <>
        <HarnessLogo id={id} />
        <HarnessLogo id={id} size={32} />
      </>,
    );
    const ids = Array.from(container.querySelectorAll('[id]'), (el) => el.id);
    expect(ids).toHaveLength(DEFINED_IDS[id] * 2);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it.each(HARNESS_IDS)('resolve inside their own svg for %s', (id) => {
    const { container } = render(
      <>
        <HarnessLogo id={id} />
        <HarnessLogo id={id} />
      </>,
    );
    let refs = 0;
    for (const svg of Array.from(container.querySelectorAll('svg'))) {
      for (const el of [svg, ...Array.from(svg.querySelectorAll('*'))]) {
        for (const { value } of Array.from(el.attributes)) {
          for (const [, ref = ''] of value.matchAll(/url\(#([^)]+)\)/g)) {
            refs += 1;
            expect(svg.querySelector(`[id="${ref}"]`)).not.toBeNull();
          }
        }
      }
    }
    // A mark that defines ids must use them; a plain one must not point anywhere.
    expect(refs > 0).toBe(DEFINED_IDS[id] > 0);
  });
});

describe('colour', () => {
  it.each(HARNESS_IDS)('declares the %s tone and paints accordingly', (id) => {
    const svg = logoOf(id);
    expect(svg).toHaveAttribute('data-tone', TONE[id]);
    if (TONE[id] === 'mono') {
      // One ink for the whole mark, so the stylesheet can pin or follow the theme.
      const fills = Array.from(svg.querySelectorAll('path'), (path) => path.getAttribute('fill'));
      expect(fills.length).toBeGreaterThan(0);
      expect(new Set(fills)).toEqual(new Set(['currentColor']));
    } else {
      expect(svg.outerHTML).not.toContain('currentColor');
      expect(svg.querySelector('path[fill^="#"], path[fill^="url(#"]')).not.toBeNull();
    }
  });
});
