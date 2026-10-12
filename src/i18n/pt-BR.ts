// The only catalog today (ADR D10): a second language is another file with the same keys.
import { errorDetailMessages } from './messages/errorDetails';
import { errorMessages } from './messages/errors';
import { historyMessages } from './messages/history';
import { noticeMessages } from './messages/notices';
import { settingsMessages } from './messages/settings';
import { setupMessages } from './messages/setup';
import { shellMessages } from './messages/shell';
import { windowMessages } from './messages/windows';

export const partials = [
  shellMessages,
  windowMessages,
  errorMessages,
  errorDetailMessages,
  noticeMessages,
  setupMessages,
  settingsMessages,
  historyMessages,
] as const;

export const ptBR = {
  ...shellMessages,
  ...windowMessages,
  ...errorMessages,
  ...errorDetailMessages,
  ...noticeMessages,
  ...setupMessages,
  ...settingsMessages,
  ...historyMessages,
} as const;

export type MessageKey = keyof typeof ptBR;
