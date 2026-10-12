// ErrorCode -> pt-BR message + exactly one action. The raw error text never reaches the UI.
import { hasMessage, t } from '../i18n';
import type { ErrorCode, HarnessError, HarnessId } from '../lib/contract';
import { errorDetails, reasonWording } from './errorDetail';
import { own } from './own';

export type ErrorAction = 'retry' | 'settings' | 'console' | 'reinstall';

const ACTIONS: Record<ErrorCode, ErrorAction> = {
  internal: 'retry',
  timeout: 'retry',
  network: 'retry',
  bad_response: 'retry',
  response_too_large: 'retry',
  rate_limited: 'retry',
  unauthorized: 'settings',
  not_found: 'settings',
  server_error: 'retry',
  tls_error: 'settings',
  invalid_url: 'settings',
  invalid_input: 'settings',
  settings_unreadable: 'retry',
  install_failed: 'reinstall',
  unsupported_plan: 'settings',
  oauth_expired: 'retry',
  binary_not_found: 'settings',
  spawn_failed: 'settings',
  handshake_failed: 'retry',
  protocol_error: 'retry',
  not_signed_in: 'retry',
  not_running: 'retry',
  endpoint_changed: 'retry',
};

// A reason can change what the user should do next: the action of the code is wrong then.
const REASON_ACTIONS: Readonly<Record<string, ErrorAction>> = {
  'unauthorized:no_token': 'retry',
  'endpoint_changed:not_an_object': 'settings',
  'endpoint_changed:no_record': 'settings',
  'endpoint_changed:id_missing': 'settings',
  'endpoint_changed:id_mismatch': 'settings',
  'install_failed:foreign_status_line': 'retry',
};

// A status code is three digits; anything else from the wire is ignored instead of printed.
const HTTP_STATUS = /^\d{3}$/;

export interface ErrorText {
  title: string;
  body: string;
  action: ErrorAction;
}

/** OpenCode rejects valid keys at times: 401/403 never claims the key is certainly wrong. */
export function isAmbiguousAuth(error: HarnessError, harness?: HarnessId): boolean {
  return (
    error.code === 'unauthorized' && (harness === 'opencode' || error.params.ambiguous === 'true')
  );
}

function actionOf(error: HarnessError): ErrorAction {
  const byReason = own(REASON_ACTIONS, `${error.code}:${error.params.reason}`);
  return byReason ?? ACTIONS[error.code] ?? 'retry';
}

export function describeError(error: HarnessError, harness?: HarnessId): ErrorText {
  const ambiguous = isAmbiguousAuth(error, harness);
  const base = ambiguous ? 'error.unauthorized_ambiguous' : `error.${error.code}`;
  const wording = ambiguous ? null : reasonWording(error);
  const titleKey = `${base}.title`;
  const bodyKey = `${base}.body`;
  const generic = {
    title: hasMessage(titleKey) ? t(titleKey) : t('error.internal.title'),
    body: hasMessage(bodyKey) ? t(bodyKey) : t('error.internal.body'),
  };
  // The status stays last: it is the line a person reads out when asking for help.
  const sentences = [wording?.body ?? generic.body, ...errorDetails(error)];
  const http = error.params.httpStatus;
  if (http !== undefined && HTTP_STATUS.test(http)) {
    sentences.push(t('error.http', { httpStatus: http }));
  }
  return {
    title: wording?.title ?? generic.title,
    body: sentences.join(' '),
    action: ambiguous ? 'console' : actionOf(error),
  };
}

/** The settings field a rejected save belongs to (`invalid_url`, `invalid_input` + params.field). */
export function fieldOf(error: HarnessError): string | null {
  if (error.code === 'invalid_url') return error.params.field ?? 'baseUrl';
  if (error.code === 'invalid_input') return error.params.field ?? null;
  return null;
}
