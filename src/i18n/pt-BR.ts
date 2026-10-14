// The only catalog today (ADR D10): a second language is another file with the same keys.

import { draftMessages } from './messages/draft';
import { errorDetailMessages } from './messages/errorDetails';
import { errorMessages } from './messages/errors';
import { historyMessages } from './messages/history';
import { noticeMessages } from './messages/notices';
import { settingsMessages } from './messages/settings';
import { setupMessages } from './messages/setup';
import { shellMessages } from './messages/shell';
import { updateMessages } from './messages/update';
import { windowMessages } from './messages/windows';

export const partials = [
  draftMessages,
  updateMessages,
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
  ...draftMessages,
  ...updateMessages,
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
