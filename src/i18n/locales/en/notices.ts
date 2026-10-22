// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    'Command installed. Limits arrive after the first reply in a Claude Code session.',
  'notice.code.claude_no_rate_limits':
    'Claude Code does not report limits (API key billing or a plan without limits).',
  'notice.code.claude_shim_outdated':
    'The command installed in the Claude Code status line was not found. Reinstall it to resume reading.',
  'notice.code.claude_oauth_partial':
    'Per-model usage (claude /usage or the account usage API) is on, but it failed ({reason}). Showing only the status line data.',
  'notice.code.codex_rollout_fallback':
    'The codex app-server did not respond ({reason}). Reading the last recorded session; the numbers may be old.',
  'notice.code.codex_no_limits': 'The Codex account does not report limits (API key login).',
  'notice.code.antigravity_pool_unknown':
    'The {pool} quota returned no data. It shows as unknown, not as zero.',
  'notice.code.litellm_no_budget': 'This key, user or team has no budget cap.',
  'notice.code.litellm_level_unreadable':
    'Could not read the {level} budget ({reason}). There may be a cap there that does not show here.',
  'notice.code.litellm_windows_truncated':
    'There are more budget windows than fit here. The least used were left out: {dropped}.',
  'notice.code.opencode_window_idle':
    'The {window} window is at 0%: it only starts counting on the next request.',
  'notice.code.opencode_rate_limited': 'The {window} window is rate-limited.',
  'notice.code.opencode_no_key':
    'Showing only the local estimate. Paste the Go API key to see the real windows.',
  'notice.code.opencode_local_estimate':
    'Local values are an estimate at list price, from this PC only. They are not the quota.',
  'notice.code.opencode_db_unreadable': 'Could not read the local OpenCode database: {reason}.',
  'notice.unknown': 'Untranslated notice: {code}',
  'notice.action.reinstall': 'Reinstall',
  'notice.action.settings': 'Enter the key',
  'notice.param.window.rolling': '5-hour',
  'notice.param.window.weekly': 'weekly',
  'notice.param.window.monthly': 'monthly',
  'notice.param.level.key': 'key',
  'notice.param.level.user': 'user',
  'notice.param.level.team': 'team',
  'notice.param.dropped.one': '{n} window',
  'notice.param.dropped.other': '{n} windows',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': 'the data folder path is invalid',
  'notice.reason.opencode_db_unreadable.not_found': 'the database file was not found',
  'notice.reason.opencode_db_unreadable.permission': 'no permission to open the database file',
  'notice.reason.opencode_db_unreadable.locked': 'the database is in use by another program',
  'notice.reason.opencode_db_unreadable.not_a_database': 'the file found is not a database',
  'notice.reason.opencode_db_unreadable.schema': 'the database format has changed',
  'notice.reason.opencode_db_unreadable.timeout': 'reading the database took too long',
  'notice.reason.opencode_db_unreadable.other': 'unidentified reason',
  'notice.severity.info': 'Info',
  'notice.severity.warn': 'Warning',
  'notice.severity.error': 'Error',
} as const satisfies Record<string, string>;
