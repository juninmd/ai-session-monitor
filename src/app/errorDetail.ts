// What the params of a failure add to its generic message. Only closed vocabularies and plain
// numbers are worded: an unknown value, `host` and the diagnostics (`phase`, `rpcCode`, `code`)
// never reach the screen.
import { hasMessage, t } from '../i18n';
import { formatInterval } from '../i18n/format';
import type { HarnessError } from '../lib/contract';

export interface ReasonWording {
  /** Set only when the reason changes what kind of failure this is. */
  title: string | null;
  body: string;
}

/** A `reason` the catalog words for this code replaces the generic body, and sometimes the title. */
export function reasonWording(error: HarnessError): ReasonWording | null {
  const reason = error.params.reason;
  if (reason === undefined) return null;
  const key = `error.${error.code}.reason.${reason}`;
  const body = `${key}.body`;
  if (!hasMessage(body)) return null;
  const title = `${key}.title`;
  return { title: hasMessage(title) ? t(title) : null, body: t(body) };
}

// The client keeps a Retry-After to an hour; anything but plain seconds is not worded.
const SECONDS = /^[1-9]\d{0,3}$/;

/** Sentences that follow the body, in order. */
export function errorDetails(error: HarnessError): string[] {
  const { code, params } = error;
  const details: string[] = [];
  if (code === 'bad_response' && params.redirect === 'true') details.push(t('error.redirect'));
  const wait = params.retryAfterSecs;
  if (code === 'rate_limited' && wait !== undefined && SECONDS.test(wait)) {
    details.push(t('error.retry_after', { wait: formatInterval(Number(wait)) }));
  }
  const field = params.field;
  const fieldKey = `field.${field}`;
  if (code === 'invalid_input' && field !== undefined && hasMessage(fieldKey)) {
    details.push(t('error.field_hint', { field: t(fieldKey) }));
  }
  return details;
}
