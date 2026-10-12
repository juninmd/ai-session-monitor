import { type MessageKey, ptBR } from './pt-BR';

export type { MessageKey } from './pt-BR';

export type MessageParams = Readonly<Record<string, string | number>>;

// A placeholder without a parameter renders as a dash: never a raw "{name}" in the UI.
const MISSING = '—';
const PLACEHOLDER = /\{(\w+)\}/g;

/** Looks up a message and fills `{name}` placeholders. */
export function t(key: MessageKey, params?: MessageParams): string {
  const template: string = ptBR[key];
  return template.replace(PLACEHOLDER, (_match, name: string) => {
    const value = params?.[name];
    return value === undefined ? MISSING : String(value);
  });
}

export function hasMessage(key: string): key is MessageKey {
  return Object.hasOwn(ptBR, key);
}
