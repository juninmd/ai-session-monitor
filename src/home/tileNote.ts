// What a card says when its source is not simply fine: why it failed, why it is disconnected and
// how old its numbers are. The wording comes from the error and notice catalogs; nothing here
// invents a reason.
import type { ViewContext } from '../app/model/windowModel';
import { describeError } from '../app/text/errorText';
import { noticeErrorCode, noticeText } from '../app/text/noticeText';
import { t } from '../i18n';
import { formatAge } from '../i18n/format';
import type { ErrorCode, HarnessView } from '../lib/contract';
import { lostSetup } from './model';

/** A reading older than this says its age on the card even when nothing failed. */
const OLD_AFTER_SECS = 600;

/** Failures a person fixes by signing in again, in the program the source reads. */
const SIGN_IN: ReadonlySet<ErrorCode> = new Set(['oauth_expired', 'not_signed_in', 'unauthorized']);

function lostNotice(view: HarnessView) {
  return lostSetup(view) ? view.notices.find((notice) => notice.severity === 'warn') : undefined;
}

/** The few words beside the dot: the reason, never a bare "failed". */
export function brokenShort(view: HarnessView): string {
  if (view.error !== null) return describeError(view.error, view.id).title;
  if (lostSetup(view)) return t('triage.short.setup');
  return t('triage.short.stale');
}

/** The whole sentence, for the screen reader and the tooltip. */
export function brokenVerdict(view: HarnessView, ctx: ViewContext): string {
  if (view.error !== null) return describeError(view.error, view.id).title;
  const lost = lostNotice(view);
  if (lost !== undefined) return noticeText(lost);
  return view.readAt === null
    ? t('surface.no_reading')
    : t('triage.verdict.stale_age', { age: formatAge(ctx.nowSecs - view.readAt) });
}

/** What to do about it, under the reason: the catalog's own sentence. */
export function brokenHint(view: HarnessView): string | null {
  if (view.error !== null) return describeError(view.error, view.id).body;
  const lost = lostNotice(view);
  return lost === undefined ? null : noticeText(lost);
}

/** A source that reads fine but whose optional extra failed on the login says so on the card. */
export function signInNote(view: HarnessView): string {
  for (const notice of view.notices) {
    const code = noticeErrorCode(notice);
    if (code !== null && SIGN_IN.has(code)) return t(`error.${code}.title`);
  }
  return '';
}

/** "Read 12 min ago", once the numbers are old or not from a recent poll. */
export function readAge(view: HarnessView, ctx: ViewContext): string | null {
  if (view.state !== 'ready' || view.readAt === null) return null;
  const age = ctx.nowSecs - view.readAt;
  return view.stale || age >= OLD_AFTER_SECS ? t('meta.read_ago', { age: formatAge(age) }) : null;
}
