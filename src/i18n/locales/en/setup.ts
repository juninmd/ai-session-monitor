// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': 'Not now',
  'consent.details': 'What will be read',

  'claude.connect.title': 'Connect to Claude Code',
  'claude.connect.lead':
    'The monitor adds a small command to the Claude Code status line. It stores only the limit percentages (5 hours and week) in a local file. No login is read.',
  'claude.connect.detail':
    'Edits the Claude Code settings.json, with a backup, and chains your current status line. Writes only the limits to %LOCALAPPDATA%. No network and no tokens. Works on Pro and Max accounts. Numbers appear after the first reply in a session and only update while Claude Code is open.',
  'claude.connect.allow': 'Install command',
  'claude.connect.tip':
    'Tip: Windows hides new icons in the overflow arrow menu. Drag the monitor icon onto the taskbar to keep it always visible.',

  'codex.connect.title': 'Connect to Codex',
  'codex.connect.lead':
    'The monitor starts the codex app-server installed here and asks how much is left of the 5-hour window and the week.',
  'codex.connect.detail':
    "Runs the codex program on your machine, read-only. No data leaves this computer and the app does not touch Codex's login file. OpenAI marks the app-server as experimental, so the format may change between versions.",
  'codex.connect.allow': 'Allow reading',

  'antigravity.connect.title': 'Connect to Antigravity',
  'antigravity.connect.lead':
    "The quota comes from the installed agy (CLI) or the IDE's local server. This route is unofficial and may stop working when Antigravity updates.",
  'antigravity.connect.detail':
    "With your permission, the monitor can run the installed agy only to read /usage (without spending tokens or reading login data) and/or read the IDE server's command line to get the local session token, talking only to 127.0.0.1. Nothing is written to disk or sent to the internet.",
  'antigravity.connect.allow': 'Allow local reading',

  'shim.title': 'Status line command',
  'shim.state.not_installed': 'Not installed',
  'shim.state.installed': 'Installed',
  'shim.state.installed_chained': 'Installed, chained with your status line',
  'shim.state.foreign': 'Another status line in use',
  'shim.state.outdated': 'Outdated: the executable was not found',
  'shim.state.unreadable': 'settings.json unreadable',
  'shim.hint.foreign':
    'When you install, your current status line keeps working: the monitor chains it.',
  'shim.hint.unreadable':
    'The monitor does not touch a file it cannot read. Fix settings.json and check again.',
  'shim.path': 'File: {path}',
  'shim.install': 'Install command',
  'shim.reinstall': 'Reinstall command',
  'shim.uninstall': 'Remove command',
  'shim.recheck': 'Check again',
  'shim.loading': 'Checking Claude Code',

  'secret.saved': 'Key saved',
  'secret.not_saved': 'No key saved',
  'secret.placeholder': 'Paste the key',
  'secret.placeholder_keep': 'Leave blank to keep the saved key',
  'secret.hint': 'Stays in Windows Credential Manager and never appears in the interface.',
  'secret.save': 'Save key',
  'secret.clear': 'Remove key',
  'secret.empty': 'Paste the API key.',
  'secret.saved_ok': 'Key saved.',
  'secret.cleared_ok': 'Key removed.',

  'opencode.title': 'Connect to OpenCode Go',
  'opencode.lead':
    'Paste your OpenCode Go API key to read the real windows (5 hours, week and month). Without the key, you can only see a local usage estimate.',
  'opencode.key_label': 'OpenCode Go API key',
  'opencode.console': 'Open the OpenCode Go console',
  'opencode.billing': 'View Zen billing',
  'opencode.zen_note': 'The Zen (prepaid) balance has no API: check the billing dashboard.',
  'opencode.local_label': 'Local estimate (this PC only)',
  'opencode.local_hint':
    'Opens the OpenCode database in read-only mode and sums only cost and tokens, at list price. It is not the real quota.',
  'opencode.data_dir': 'OpenCode data folder',
  'opencode.data_dir_hint': 'Leave blank to use the default folder.',
  'opencode.save_settings': 'Save folder',

  'kimi.title': 'Connect to Kimi Code',
  'kimi.lead':
    'Paste your Kimi Code API key to read the plan quota: the 5-hour window, the week and the month total, when the plan reports them.',
  'kimi.detail':
    "Unofficial reading: the usage API is undocumented and may change without notice. The key is sent only to api.kimi.com and stays in Windows Credential Manager. No cookie or login from another tool is used. A key from Moonshot's open platform is a different product and does not work here.",
  'kimi.key_label': 'Kimi Code API key',
  'kimi.console': 'Open the Kimi Code console',
  'kimi.note':
    'Unofficial reading of the Kimi Code usage API. Week and month only appear when the plan reports them.',

  'zai.title': 'Connect to z.ai',
  'zai.lead':
    'Paste your GLM Coding plan API key to read the 5-hour window, the week and the monthly MCP tools quota.',
  'zai.detail':
    'Unofficial reading: the quota API is undocumented and may change without notice. The key is sent only to api.z.ai and stays in Windows Credential Manager. No cookie or login from another tool is used. BigModel accounts (open.bigmodel.cn) are not read.',
  'zai.key_label': 'z.ai API key',
  'zai.console': 'Open the z.ai keys',
  'zai.note':
    'Unofficial reading of the GLM Coding plan quota API, on the global server (api.z.ai).',

  'openrouter.title': 'Connect to OpenRouter',
  'openrouter.lead':
    'Paste your OpenRouter API key to see the key spending limit, usage and, when the key allows it, the credit balance.',
  'openrouter.detail':
    'Official reading, through the documented OpenRouter API. The key is sent only to openrouter.ai and stays in Windows Credential Manager. The credit balance requires a management key; with a regular key, only the key limit and usage appear.',
  'openrouter.key_label': 'OpenRouter API key',
  'openrouter.console': 'Open the OpenRouter keys',
  'openrouter.note':
    'Official reading of the OpenRouter API. Without a limit on the key there is no percentage, only usage. The balance only appears with a management key.',

  'deepseek.title': 'Connect to DeepSeek',
  'deepseek.lead':
    'Paste your DeepSeek API key to see the account balance, in each currency where it has credit.',
  'deepseek.detail':
    'Official reading, through the documented DeepSeek API. The key is sent only to api.deepseek.com and stays in Windows Credential Manager. The balance has no limit or reset time, so there is no percentage; yuan and dollars are never added together.',
  'deepseek.key_label': 'DeepSeek API key',
  'deepseek.console': 'Open the DeepSeek keys',
  'deepseek.note':
    'Official reading of the DeepSeek balance API. It is money, not a quota: there is no percentage or reset.',

  'deepinfra.title': 'Connect to DeepInfra',
  'deepinfra.lead':
    "Paste your DeepInfra API key to see the balance, the spending limit and this month's spending.",
  'deepinfra.detail':
    'Official reading, through the DeepInfra billing API. The key is sent only to api.deepinfra.com and stays in Windows Credential Manager. Without a spending limit there is no percentage, only dollar amounts.',
  'deepinfra.key_label': 'DeepInfra API key',
  'deepinfra.console': 'Open the DeepInfra keys',
  'deepinfra.note':
    'Official reading of the DeepInfra billing API. Values are read as dollars; without a spending limit there is no percentage.',

  'moonshot.title': 'Connect to Moonshot',
  'moonshot.lead':
    'Paste your Moonshot platform (platform.moonshot.ai) API key to see the account balance.',
  'moonshot.detail':
    "Official reading, through Moonshot's documented balance API. The key is sent only to api.moonshot.ai and stays in Windows Credential Manager. Keys from the kimi.com platform are separate and do not work here. The balance is money: there is no percentage or reset.",
  'moonshot.key_label': 'Moonshot API key',
  'moonshot.console': 'Open the Moonshot keys',
  'moonshot.note':
    'Official reading of the Moonshot balance API, in dollars. The cash balance can go negative when an amount is owed.',

  'novita.title': 'Connect to Novita',
  'novita.lead':
    'Paste your Novita API key to see the account balance, the credit limit and pending charges.',
  'novita.detail':
    "Official reading, through Novita's documented billing API. The key is sent only to api.novita.ai and stays in Windows Credential Manager. The balance is money: there is no percentage or reset.",
  'novita.key_label': 'Novita API key',
  'novita.console': 'Open the Novita keys',
  'novita.note':
    'Official reading of the Novita balance API, in dollars. The balance can go negative when an amount is owed.',

  'ollama.title': 'Connect to Ollama Cloud',
  'ollama.lead':
    "Paste an Ollama Cloud API key to see the plan's included credits and the purchased balance.",
  'ollama.detail':
    "Official reading, through the documented Ollama Cloud balance API. Create the key at ollama.com/settings/keys: the CLI's subscription key is never read. The key is sent only to ollama.com and stays in Windows Credential Manager. The response format depends on the plan (monthly credits or session and week windows).",
  'ollama.key_label': 'Ollama Cloud API key',
  'ollama.console': 'Open the Ollama Cloud keys',
  'ollama.note':
    'Official reading of the Ollama Cloud balance API. Current plans show dollar credits; older plans show session and week percentages.',

  'minimax.title': 'Connect to MiniMax',
  'minimax.lead':
    'Paste your MiniMax Token Plan key to see per-model quotas: the current interval and the week.',
  'minimax.detail':
    'Unofficial reading: the Token Plan quota route has no published schema and may change without notice. The key is sent only to api.minimax.io (global server) and stays in Windows Credential Manager. Compare the numbers with the MiniMax console the first time.',
  'minimax.key_label': 'MiniMax Token Plan key',
  'minimax.console': 'Open the MiniMax keys',
  'minimax.note':
    'Unofficial reading of the MiniMax Token Plan quota, per model. A model with no quota in the plan does not appear.',

  'litellm.title': 'Connect to LiteLLM',
  'litellm.title_edit': 'LiteLLM connection',
  'litellm.url': 'Server URL',
  'litellm.url_placeholder': 'https://litellm.yourcompany.com',
  'litellm.url_hint': 'HTTPS is required. HTTP is only accepted for localhost.',
  'litellm.url_required': 'Enter the server URL.',
  'litellm.url_invalid': 'Enter a full URL, like https://litellm.yourcompany.com.',
  'litellm.url_http': 'Use https://. HTTP only works for localhost.',
  'litellm.url_parts': 'Remove the username, password, query parameters and # from the URL.',
  'litellm.key': 'API key',
  'litellm.key_required': 'Paste the LiteLLM API key.',
  'litellm.key_rebind':
    'The saved key only applies to the current connection. Since it changed, paste the key again.',
  'litellm.scope': 'What to measure',
  'litellm.scope.key': 'This key',
  'litellm.scope.user': 'User',
  'litellm.scope.team': 'Team',
  'litellm.entity.user': 'User ID',
  'litellm.entity.team': 'Team ID',
  'litellm.entity_required': 'Enter the ID.',
  'litellm.advanced': 'Advanced',
  'litellm.auth_header': 'Authentication header',
  'litellm.auth_header_hint': 'Change only if your proxy uses its own header for the key.',
  'litellm.auth_header_invalid': 'Use only letters, numbers and hyphens.',
  'litellm.poll': 'Refresh every',
  'litellm.timeout': 'Timeout',
  'litellm.tls': 'TLS certificate',
  'litellm.tls.system': 'From Windows',
  'litellm.tls.custom_ca': 'Custom CA',
  'litellm.tls.insecure': 'Do not verify',
  'litellm.pem': 'CA file (.pem)',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': 'Enter the path to the .pem file.',
  'litellm.insecure_host': 'Server without verification',
  'litellm.insecure_host_hint': 'The certificate is skipped only for this server.',
  'litellm.insecure_host_required': 'Enter the server.',
  'litellm.insecure_warning':
    'The connection will not be verified and the key could be intercepted. Use only on a trusted network.',
  'litellm.insecure_confirm': "I understand the risk and want to ignore this server's certificate.",
  'litellm.insecure_confirm_required': 'Confirm to continue without verifying the certificate.',
  'litellm.save': 'Save and connect',
  'litellm.saving': 'Saving…',
  'litellm.saved': 'Settings saved. Reading the server…',
  'litellm.edit': 'Edit connection',
  'litellm.delay_note':
    'Spend and reset can lag by up to about 10 min on the server. The cap may be on the user or the team, not just the key.',
} as const satisfies Record<string, string>;
