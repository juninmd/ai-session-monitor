// One title and one body for every ErrorCode (src/lib/contract/view.ts), plus the OpenCode
// ambiguity wording. src/i18n/catalog.test.ts fails when a code is missing.
export const errorMessages = {
  'error.internal.title': 'Something went wrong',
  'error.internal.body':
    'An internal error occurred while reading this source. Try again; if it keeps happening, restart the monitor.',
  'error.timeout.title': 'The reading took too long',
  'error.timeout.body': 'The source did not respond in time. Try again in a moment.',
  'error.network.title': 'Cannot reach the source',
  'error.network.body': 'Could not talk to the server. Check your network and try again.',
  'error.bad_response.title': 'Unexpected response',
  'error.bad_response.body':
    'The source replied with something the monitor does not understand. Try again later.',
  'error.response_too_large.title': 'Response too large',
  'error.response_too_large.body':
    'The response exceeded the accepted size and was discarded for safety.',
  'error.rate_limited.title': 'Too many requests',
  'error.rate_limited.body':
    'The source asked for a short wait. The monitor will retry on its own, with a longer interval.',
  'error.unauthorized.title': 'Key or login rejected',
  'error.unauthorized.body':
    'The server rejected the credentials. Check the key and its scope in settings.',
  'error.unauthorized_ambiguous.title': 'Could not verify the key or the Go subscription',
  'error.unauthorized_ambiguous.body':
    'The service rejected the request. This happens with valid keys and also without a Go subscription. Check the key and the subscription in the OpenCode console.',
  'error.unauthorized_ambiguous.kimi.title': 'Could not verify the Kimi Code key',
  'error.unauthorized_ambiguous.kimi.body':
    'The service rejected the request. This happens with expired keys and also with a key from the Moonshot open platform, which is a different product. Check in the Kimi Code console that the key belongs to the plan.',
  'error.unauthorized_ambiguous.zai.title': 'Could not verify the z.ai key',
  'error.unauthorized_ambiguous.zai.body':
    'The service rejected the request. This happens with valid keys and also with accounts without the GLM Coding plan. Check the key and the plan in the z.ai dashboard.',
  'error.not_found.title': 'Address not found',
  'error.not_found.body':
    'The server does not recognize the path or ID provided. Check the URL, the scope and the ID.',
  'error.server_error.title': 'Server error',
  'error.server_error.body': 'The source server failed. Try again later.',
  'error.tls_error.title': 'TLS certificate failure',
  'error.tls_error.body':
    'The server certificate was not accepted. Choose the right CA in settings.',
  'error.invalid_url.title': 'Invalid URL',
  'error.invalid_url.body':
    'Use https://. HTTP only works for localhost, and the URL cannot contain a username, password, query parameters or #.',
  'error.invalid_input.title': 'Invalid value',
  'error.invalid_input.body': 'Fix the indicated field and try again.',
  'error.settings_unreadable.title': 'Settings file unreadable',
  'error.settings_unreadable.body':
    'Could not read settings.json. Nothing was changed. Fix the file and try again.',
  'error.install_failed.title': 'Installation failed',
  'error.install_failed.body':
    'Could not install the command in the Claude Code status line. Nothing was changed.',
  'error.unsupported_plan.title': 'Plan has no limits to read',
  'error.unsupported_plan.body':
    'The account uses a plan or an API key that does not report limits. You can turn this source off.',
  'error.oauth_expired.title': 'Claude Code login expired',
  'error.oauth_expired.body': 'Open Claude Code to renew the login and try again.',
  'error.binary_not_found.title': 'Program not found',
  'error.binary_not_found.body':
    'The monitor could not find the executable. Install the program or set its path in settings.',
  'error.spawn_failed.title': 'Could not start the program',
  'error.spawn_failed.body': 'The process did not start. Check the executable path.',
  'error.handshake_failed.title': 'Could not talk to the program',
  'error.handshake_failed.body':
    'The program started but did not respond as expected. Update it and try again.',
  'error.protocol_error.title': 'Unexpected format',
  'error.protocol_error.body': 'The data format has changed. Update the monitor or the program.',
  'error.not_signed_in.title': 'Account not connected',
  'error.not_signed_in.body': 'Sign in to your account in the program and try again.',
  'error.not_running.title': 'Program closed',
  'error.not_running.body': 'This reading needs the program to be open. Open it and try again.',
  'error.endpoint_changed.title': 'The source changed format',
  'error.endpoint_changed.body':
    'The service changed and this reading no longer works as before. The monitor may need an update.',
  'error.http': 'HTTP code: {httpStatus}.',
} as const satisfies Record<string, string>;
