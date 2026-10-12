// Helpers for tests that audit SVG markup: parse, allowlist and a structural fingerprint.
// The allowlist is deliberately small: a new element or attribute must be added here on purpose.
const ELEMENTS = new Set([
  'svg',
  'title',
  'g',
  'path',
  'defs',
  'mask',
  'linearGradient',
  'stop',
  'filter',
  'feFlood',
  'feBlend',
  'feGaussianBlur',
]);

const ATTRIBUTES = new Set([
  // root and wrapper
  'xmlns',
  'viewBox',
  'class',
  'width',
  'height',
  'aria-hidden',
  'focusable',
  'data-logo',
  'data-tone',
  // geometry and paint
  'd',
  'fill',
  'fill-rule',
  'clip-rule',
  'fill-opacity',
  'x',
  'y',
  // masks, gradients and filters, all referenced by local id
  'id',
  'mask',
  'filter',
  'maskUnits',
  'filterUnits',
  'gradientUnits',
  'x1',
  'x2',
  'y1',
  'y2',
  'offset',
  'stop-color',
  'color-interpolation-filters',
  'flood-opacity',
  'result',
  'in',
  'in2',
  'stdDeviation',
]);

const SVG_NS = 'http://www.w3.org/2000/svg';

export function parseSvg(source: string): Element {
  const doc = new DOMParser().parseFromString(source, 'image/svg+xml');
  if (doc.getElementsByTagName('parsererror').length > 0) throw new Error('not well-formed XML');
  return doc.documentElement;
}

const allElements = (root: Element): Element[] => [root, ...Array.from(root.querySelectorAll('*'))];

/** Everything in `root` that is not plain vector markup: scripts, styles, handlers, remote refs. */
export function unsafeFindings(root: Element): string[] {
  const found: string[] = [];
  if (root.localName !== 'svg') found.push(`root is <${root.localName}>`);
  for (const el of allElements(root)) {
    const tag = el.localName;
    if (!ELEMENTS.has(tag)) found.push(`element <${tag}>`);
    for (const { name, value } of Array.from(el.attributes)) {
      if (!ATTRIBUTES.has(name)) found.push(`attribute ${tag}@${name}`);
      if (name === 'xmlns' && value !== SVG_NS) found.push(`namespace ${value}`);
      if (name !== 'xmlns' && /javascript:|data:|https?:|\/\//i.test(value)) {
        found.push(`${tag}@${name} reaches outside the document`);
      }
      for (const [, target = ''] of value.matchAll(/url\(([^)]*)\)/g)) {
        if (!/^#[\w-]+$/.test(target)) found.push(`${tag}@${name} references ${target}`);
      }
    }
  }
  return found;
}

/**
 * One line per element (title aside): tag and attributes, with ids replaced by their order of
 * appearance, so a vendored file and a rendered component compare byte for byte on path data.
 * Only the root's viewBox counts; sizing and classes belong to the component.
 */
export function fingerprint(root: Element): string[] {
  const nodes = allElements(root).filter((el) => el.localName !== 'title');
  const ids = nodes.flatMap((el) => el.getAttribute('id') ?? []);
  const canon = (value: string) =>
    value.replace(/url\(#([^)]+)\)/g, (_, id: string) => `url(#${ids.indexOf(id)})`);
  return nodes.map((el) => {
    const attrs = Array.from(el.attributes)
      .filter(({ name }) => el !== root || name === 'viewBox')
      .map(
        ({ name, value }) => `${name}=${name === 'id' ? `#${ids.indexOf(value)}` : canon(value)}`,
      )
      .sort();
    return `${el.localName} ${attrs.join(' ')}`;
  });
}
