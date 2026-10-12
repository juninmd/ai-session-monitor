// Maps a rejected save to the message printed next to the field it belongs to.
import { t } from '../i18n';
import type { HarnessError } from '../lib/contract';
import { describeError } from './errorText';
import { own } from './own';

const FIELD_MESSAGES: Readonly<Record<string, () => string>> = {
  value: () => t('secret.empty'),
  baseUrl: () => t('litellm.url_invalid'),
  entityId: () => t('litellm.entity_required'),
  authHeader: () => t('litellm.auth_header_invalid'),
  pemPath: () => t('litellm.pem_required'),
  host: () => t('litellm.insecure_host_required'),
};

/** Field-specific wording when the backend names the field, the generic body otherwise. */
export function fieldMessage(error: HarnessError, field: string | null): string {
  const specific = own(FIELD_MESSAGES, field);
  if (error.code === 'invalid_url') return describeError(error).body;
  if (specific !== undefined) return specific();
  const text = describeError(error);
  return `${text.title}. ${text.body}`;
}
