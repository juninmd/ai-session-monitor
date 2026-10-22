// bn: translated from the English catalog. Every locale folder holds the same partial
// files and exposes the same keys; English is the fallback for a key a locale has not got yet.

import { appMessages } from './app';
import { componentsMessages } from './components';
import { draftMessages } from './draft';
import { errorDetailMessages } from './errorDetails';
import { errorMessages } from './errors';
import { flyoutMessages } from './flyout';
import { historyMessages } from './history';
import { noticeMessages } from './notices';
import { settingsMessages } from './settings';
import { setupMessages } from './setup';
import { shellMessages } from './shell';
import { surfacesMessages } from './surfaces';
import { updateMessages } from './update';
import { windowMessages } from './windows';

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
  surfacesMessages,
  flyoutMessages,
  appMessages,
  componentsMessages,
] as const;

export const bn = {
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
  ...surfacesMessages,
  ...flyoutMessages,
  ...appMessages,
  ...componentsMessages,
} as const;
