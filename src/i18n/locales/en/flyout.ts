// English catalog: flyout (onboarding, command palette, provider detail, history board).
export const flyoutMessages = {
  // Onboarding wizard
  'onboarding.step.choose': 'Choose',
  'onboarding.step.connect': 'Connect',
  'onboarding.step.verify': 'Verify',
  'onboarding.step.done': 'Done',
  'onboarding.choose.title': 'What do you want to track?',
  'onboarding.choose.note':
    'Checked items were found on this machine. Nothing is read until you continue.',
  'onboarding.tag.responds': 'responds',
  'onboarding.tag.silent': 'no reading',
  'onboarding.tag.needs_key': 'needs a key',
  'onboarding.tag.needs_consent': 'needs permission',
  'onboarding.connect.title': 'Credentials and permissions',
  'onboarding.connect.consent_note':
    'This source reads files or processes on this PC. Nothing is read until you say OK.',
  'onboarding.connect.allow': 'Allow reading',
  'onboarding.connect.details': 'See details',
  'onboarding.connect.litellm_note': 'Needs the proxy address, the scope and the key.',
  'onboarding.connect.multi_note':
    'There is more than one way to connect: a key or reading the local database.',
  'onboarding.connect.configure': 'Set up',
  'onboarding.connect.nothing':
    'Nothing to connect: the chosen sources already have what they need.',
  'onboarding.verify.title': 'What each source answered',
  'onboarding.verify.reading': 'Reading…',
  'onboarding.verify.read_now': 'Read now',
  'onboarding.verify.first.one': 'one quota · the first is {pct} used',
  'onboarding.verify.first.other': '{count} quotas · the first is {pct} used',
  'onboarding.verify.back_to_key': 'Go back and paste the key',
  'onboarding.done.title': 'Monitoring {done} of {total}',
  'onboarding.done.body':
    'The panel opens sorted by urgency: whatever is closest to running out comes first. Anything that did not answer stays under "No reading", with no made-up numbers.',
  'onboarding.done.go': 'Go to the panel',
  'onboarding.done.again': 'Read again',
  'onboarding.back': 'Back',
  'onboarding.continue': 'Continue with {count}',
  'onboarding.connected': 'I have connected',
  'onboarding.finish': 'Finish',
  'onboarding.outcome.loading': 'Reading…',
  'onboarding.outcome.disabled': 'Off',
  'onboarding.outcome.needs_consent': 'Waiting for your permission',
  'onboarding.outcome.needs_setup': 'Credential missing',
  'onboarding.outcome.no_data': 'No data yet',
  'onboarding.outcome.failed': 'Reading failed',
  'onboarding.outcome.stale': 'Old reading',
  'onboarding.outcome.stale_ago': 'Old reading from {age} ago',
  'onboarding.outcome.no_windows': 'Reading values, no quotas',
  'onboarding.outcome.quotas.one': '{count} quota',
  'onboarding.outcome.quotas.other': '{count} quotas',

  // Ctrl+K palette
  'palette.group.providers': 'Providers',
  'palette.group.actions': 'Actions',
  'palette.group.settings': 'Settings',
  'palette.group.demo': 'Demo',
  'palette.refresh.label': 'Refresh all sources',
  'palette.refresh.hint': 'read again now',
  'palette.refresh.keywords': 'refresh reload update',
  'palette.copy.label': 'Copy status summary',
  'palette.copy.hint.one': '{count} source as text',
  'palette.copy.hint.other': '{count} sources as text',
  'palette.copy.keywords': 'clipboard copy status report',
  'palette.general.label': 'General settings',
  'palette.general.hint': 'display, limits, tray',
  'palette.general.keywords': 'preferences options',
  'palette.configure.label': 'Set up {name}',
  'palette.configure.hint_stale': 'old reading',
  'palette.configure.hint_default': 'source and credentials',
  'palette.configure.keywords': 'configure set up adjust source credential',
  'palette.scenario.label': 'Scenario: {name}',
  'palette.scenario.hint': 'fictional data',
  'palette.scenario.keywords': 'mock demo data',
  'palette.close_aria': 'Close the quick command',
  'palette.dialog_aria': 'Quick command',
  'palette.input_aria': 'Search commands',
  'palette.placeholder': 'Go to provider, refresh, copy summary…',
  'palette.empty': 'Nothing found',
  'palette.foot.navigate': 'navigate',
  'palette.foot.run': 'run',
  'palette.key.esc': 'Esc',
  'palette.key.enter': 'Enter',

  // Provider actions (preview only)
  'provider_actions.outcome.reset': 'Reset simulated. Quotas and balance updated in the preview.',
  'provider_actions.outcome.noCredit':
    'No reset available. The balance changed; refresh the reading.',
  'provider_actions.outcome.nothingToReset': 'There is no quota to renew in the preview.',
  'provider_actions.outcome.unavailable': 'The action is not available for this reading.',
  'provider_actions.resets.one': '{count} reset available',
  'provider_actions.resets.other': '{count} resets available',
  'provider_actions.use_reset': 'Use reset',
  'provider_actions.confirm_aria': 'Confirm quota reset',
  'provider_actions.confirm_body':
    'Use 1 Codex reset? Using it is final. In this preview, the action is simulated.',
  'provider_actions.confirm': 'Confirm reset',
  'provider_actions.cancel': 'Cancel',
  'provider_actions.reset_failed': 'Could not simulate the reset. Try again.',
  'provider_actions.low_priority_offer': 'Low-priority offer · preview',
  'provider_actions.copy_command': 'Copy /low-priority',
  'provider_actions.low_priority_hint':
    'Use {command} in the interactive session that offered this mode.',
  'provider_actions.copied': 'Command copied. The mode still has to be turned on in Claude Code.',
  'provider_actions.copy_failed': 'Could not copy. Select the command above and copy it manually.',

  // History board and trend
  'history.board.no_readable': "None of this source's quotas keeps a reading with a renewal time.",
  'history.board.failed': 'Could not read the history.',
  'history.board.loading': 'Loading the readings…',
  'history.board.empty': 'No readings stored for this quota yet.',
  'history.trend.none': 'No reading in this window.',
  'history.trend.first': 'Peak of {peak} so far, with no earlier renewal to compare against.',
  'history.trend.compare.one':
    'Peak of {peak} so far · the previous renewal closed at {avg} on average.',
  'history.trend.compare.other':
    'Peak of {peak} so far · the previous {count} renewals closed at {avg} on average.',
  'history.trend.projection_desc':
    'The projection follows the current pace until it runs out in {duration}.',

  // Quota sheet
  'quota.status.ok': 'comfortable',
  'quota.status.warn': 'near the limit',
  'quota.status.crit': 'critical',
  'quota.status.out': 'used up',
  'quota.amount.unlimited': '{used} spent, no cap',
  'quota.amount.of': '{used} of {limit}',
  'quota.amount.free': '{free} free',
  'quota.renew.restarted': 'restarted',
  'quota.renew.restarted_at': 'restarted at {time}',
  'quota.renew.in': 'renews in {duration}',
  'quota.spoken_left': '{duration} until renewal.',
  'quota.reading.expired': 'Expired',
  'quota.reading.stale': 'Old',
  'quota.spoken_stale': '{name}: outdated reading.',
  'quota.forecast.out': 'Used up',
  'quota.forecast.risk': 'Runs out in {duration}',
  'quota.forecast.fine': 'Lasts until renewal',
  'quota.forecast.unknown': 'Not enough to forecast',

  // Provider detail drawer
  'drawer.close': 'Close the details',
  'drawer.refresh': 'Refresh {name}',
  'drawer.pin': 'Pin {name} to the top',
  'drawer.unpin': 'Unpin {name}',
  'drawer.settings': 'Set up {name}',

  // Candidate sources (not wired yet)
  'candidates.amp.credential': 'Local CLI (`amp`)',
  'candidates.amp.offer':
    '`amp usage` returns the credit balance and included usage. Same fixed-CLI pattern already used by Claude and Antigravity.',
  'candidates.kiro.credential': "The CLI's own credential",
  'candidates.kiro.offer':
    'GetUsageLimits returns a single credit pool with used, limit, percentage and reset.',
  'candidates.cline.credential': 'API key',
  'candidates.cline.offer':
    '`/users/me/plan/usage-limits`: 5 h, weekly and monthly windows with percentage and resetsAt.',
  'candidates.windsurf.credential': 'Team service key',
  'candidates.windsurf.offer':
    'GetTeamCreditBalance and GetUsageConfig through the official API. Enterprise plan only.',
  'candidates.command_code.credential': 'Local CLI (`commandcode`)',
  'candidates.command_code.offer':
    '`/alpha/billing/credits` and `/subscriptions`: 5 h and weekly spend windows, plan and monthly credits.',
} as const;
