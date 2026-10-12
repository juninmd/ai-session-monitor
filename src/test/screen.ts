// What a person (or a screen reader) is given by a rendered panel, and what must never be in it.
import type { HarnessView } from '../lib/contract';

/** Every string a person or a screen reader gets: text nodes, plus label, title and alt. */
export function readable(root: HTMLElement): string {
  const out: string[] = [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) out.push(node.nodeValue ?? '');
  for (const element of root.querySelectorAll('[aria-label],[title],[alt]')) {
    for (const name of ['aria-label', 'title', 'alt']) out.push(element.getAttribute(name) ?? '');
  }
  // Intl puts a no-break space inside "US$ 3,20"; expectations spell a plain one.
  return out.join('\n').replace(/[^\S\n]/g, ' ');
}

/** Words only the wire knows: the ids, codes and keys the view carries. */
export const wireWords = (view: HarnessView): string[] => [
  ...view.windows.map((w) => w.id),
  ...view.facts.map((f) => f.id),
  ...view.notices.map((n) => n.code),
  ...Object.keys(view.meta),
  ...(view.source ? [view.source.id] : []),
];

const escaped = (word: string) => word.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** The word as a whole: not inside a longer word, nor glued to a hyphen or underscore. */
export const standsAlone = (word: string): RegExp =>
  new RegExp(`(?<![\\p{L}\\p{N}_-])${escaped(word)}(?![\\p{L}\\p{N}_-])`, 'u');

/** The view's wire words that made it onto the screen as written. */
export const shownWireWords = (view: HarnessView, text: string): string[] =>
  wireWords(view).filter((word) => standsAlone(word).test(text));
