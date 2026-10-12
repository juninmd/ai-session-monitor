// Notice code -> pt-BR sentence. An unknown code is shown with its code, never dropped.
import { hasMessage, t } from '../i18n';
import { formatNumber } from '../i18n/format';
import type { ErrorCode, Notice } from '../lib/contract';
import { ERROR_CODES } from '../lib/contract';
import { groupName } from './layout';
import { own } from './own';

export type NoticeAction = 'reinstall' | 'settings';

// Codes and params come from collectors (some from remote servers): a long one must not flood the card.
const MAX_SHOWN = 48;
const clip = (value: string) =>
  value.length > MAX_SHOWN ? `${value.slice(0, MAX_SHOWN)}…` : value;

const ACTIONS: Readonly<Record<string, NoticeAction>> = {
  claude_shim_outdated: 'reinstall',
  opencode_no_key: 'settings',
};

const isErrorCode = (value: string): value is ErrorCode =>
  (ERROR_CODES as readonly string[]).includes(value);

// Only these notices carry an ErrorCode wire name in `reason`. Any other `reason` has its own
// vocabulary (OpenCode's `not_found` is a missing database file, not "Endereço não encontrado").
const ERROR_CODE_REASON: ReadonlySet<string> = new Set([
  'claude_oauth_partial',
  'codex_rollout_fallback',
  'litellm_level_unreadable',
]);

// A count from the wire: plain digits only, so "1e3" or "-3" never pass for a number.
const COUNT = /^\d{1,6}$/;

/** A catalog word for a wire value, or the value itself when nobody named it yet. */
function word(prefix: string, value: string): string {
  const key = `${prefix}.${value}`;
  return hasMessage(key) ? t(key) : value;
}

function reasonText(code: string, reason: string): string {
  if (ERROR_CODE_REASON.has(code)) return isErrorCode(reason) ? t(`error.${reason}.title`) : reason;
  return word(`notice.reason.${code}`, reason);
}

function countText(value: string): string {
  if (!COUNT.test(value)) return value;
  const n = Number(value);
  return t(n === 1 ? 'notice.param.dropped.one' : 'notice.param.dropped.other', {
    n: formatNumber(n),
  });
}

/** Params arrive as machine ids; the sentence wants words. */
function readableParams(notice: Notice): Record<string, string> {
  const params = Object.fromEntries(
    Object.entries(notice.params).map(([key, value]) => [key, clip(value)]),
  );
  const { window, pool, level, dropped, reason } = params;
  if (window !== undefined) params.window = word('notice.param.window', window);
  if (pool !== undefined) params.pool = groupName(pool);
  if (level !== undefined) params.level = word('notice.param.level', level);
  if (dropped !== undefined) params.dropped = countText(dropped);
  if (reason !== undefined) params.reason = reasonText(notice.code, reason);
  return params;
}

export function noticeText(notice: Notice): string {
  const key = `notice.code.${notice.code}`;
  if (!hasMessage(key)) return t('notice.unknown', { code: clip(notice.code) });
  return t(key, readableParams(notice));
}

export function noticeAction(notice: Notice): NoticeAction | null {
  return own(ACTIONS, notice.code) ?? null;
}

export function noticeKey(notice: Notice): string {
  return `${notice.code}|${JSON.stringify(Object.entries(notice.params).sort())}`;
}

/** Identical notices (same code and params) collapse into one. */
export function uniqueNotices(notices: readonly Notice[]): Notice[] {
  const seen = new Set<string>();
  return notices.filter((notice) => {
    const key = noticeKey(notice);
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
