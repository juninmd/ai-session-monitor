// Draft <-> settings mapping and client-side checks for the LiteLLM connection form. The backend
// stays the authority (it also rejects what slips through); these checks only save a round trip.
import { t } from '../i18n';
import type { LitellmScope, LitellmSettings, TlsMode } from '../lib/contract';
import { own } from './own';

export interface LitellmDraft {
  baseUrl: string;
  scope: LitellmScope;
  entityId: string;
  authHeader: string;
  pollIntervalSecs: number;
  timeoutSecs: number;
  tlsMode: TlsMode['mode'];
  pemPath: string;
  insecureHost: string;
  insecureConfirmed: boolean;
}

/** `key` is the secret input, `form` an error that belongs to no single field. */
export type DraftField =
  | 'baseUrl'
  | 'key'
  | 'entityId'
  | 'authHeader'
  | 'pemPath'
  | 'host'
  | 'confirm'
  | 'form';
export type DraftErrors = Partial<Record<DraftField, string>>;

/** Top to bottom, so the first invalid field is the one that gets focus. */
export const FIELD_ORDER: readonly DraftField[] = [
  'baseUrl',
  'key',
  'entityId',
  'authHeader',
  'pemPath',
  'host',
  'confirm',
];

/** Backend `params.field` names -> form fields. */
const BACKEND_FIELDS: Readonly<Record<string, DraftField>> = {
  baseUrl: 'baseUrl',
  value: 'key',
  entityId: 'entityId',
  authHeader: 'authHeader',
  pemPath: 'pemPath',
  host: 'host',
};

export function draftFieldOf(backendField: string | null): DraftField {
  return own(BACKEND_FIELDS, backendField) ?? 'form';
}

export function toDraft(saved: LitellmSettings): LitellmDraft {
  return {
    baseUrl: saved.baseUrl ?? '',
    scope: saved.scope,
    entityId: saved.entityId ?? '',
    authHeader: saved.authHeader,
    pollIntervalSecs: saved.pollIntervalSecs,
    timeoutSecs: saved.timeoutSecs,
    tlsMode: saved.tls.mode,
    pemPath: saved.tls.mode === 'custom_ca' ? saved.tls.pemPath : '',
    insecureHost: saved.tls.mode === 'insecure' ? saved.tls.host : '',
    // A saved insecure host was confirmed when it was saved.
    insecureConfirmed: saved.tls.mode === 'insecure',
  };
}

function tlsOf(draft: LitellmDraft): TlsMode {
  switch (draft.tlsMode) {
    case 'custom_ca':
      return { mode: 'custom_ca', pemPath: draft.pemPath.trim() };
    case 'insecure':
      return { mode: 'insecure', host: draft.insecureHost.trim() };
    case 'system':
      return { mode: 'system' };
  }
}

export function fromDraft(base: LitellmSettings, draft: LitellmDraft): LitellmSettings {
  const baseUrl = draft.baseUrl.trim();
  return {
    ...base,
    baseUrl: baseUrl === '' ? null : baseUrl,
    scope: draft.scope,
    entityId: draft.scope === 'key' ? null : draft.entityId.trim() || null,
    authHeader: draft.authHeader.trim(),
    pollIntervalSecs: draft.pollIntervalSecs,
    timeoutSecs: draft.timeoutSecs,
    tls: tlsOf(draft),
  };
}

export function isLoopbackHost(hostname: string): boolean {
  return (
    hostname === 'localhost' ||
    hostname.endsWith('.localhost') ||
    hostname === '[::1]' ||
    /^127(\.\d{1,3}){3}$/.test(hostname)
  );
}

/** The message for an unusable base URL, or null when it looks acceptable. */
export function checkBaseUrl(raw: string): string | null {
  const value = raw.trim();
  if (value === '') return t('litellm.url_required');
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return t('litellm.url_invalid');
  }
  if (url.protocol !== 'https:' && url.protocol !== 'http:') return t('litellm.url_invalid');
  if (url.protocol === 'http:' && !isLoopbackHost(url.hostname)) return t('litellm.url_http');
  if (url.username || url.password || url.search || url.hash) return t('litellm.url_parts');
  return null;
}

export function hostOf(raw: string): string {
  try {
    return new URL(raw.trim()).hostname;
  } catch {
    return '';
  }
}

const HEADER_NAME = /^[A-Za-z0-9-]+$/;

export function validateDraft(
  draft: LitellmDraft,
  key: { saved: boolean; typed: boolean },
): DraftErrors {
  const errors: DraftErrors = {};
  const url = checkBaseUrl(draft.baseUrl);
  if (url) errors.baseUrl = url;
  if (!key.saved && !key.typed) errors.key = t('litellm.key_required');
  if (draft.scope !== 'key' && draft.entityId.trim() === '') {
    errors.entityId = t('litellm.entity_required');
  }
  if (!HEADER_NAME.test(draft.authHeader.trim())) {
    errors.authHeader = t('litellm.auth_header_invalid');
  }
  if (draft.tlsMode === 'custom_ca' && draft.pemPath.trim() === '') {
    errors.pemPath = t('litellm.pem_required');
  }
  if (draft.tlsMode === 'insecure') {
    if (draft.insecureHost.trim() === '') errors.host = t('litellm.insecure_host_required');
    if (!draft.insecureConfirmed) errors.confirm = t('litellm.insecure_confirm_required');
  }
  return errors;
}
