// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    'Your current status line command is too large for the monitor to chain. Nothing was changed.',
  'error.install_failed.reason.path_unsupported.body':
    'The data folder path has characters the status line command does not accept. Nothing was changed.',
  'error.install_failed.reason.serialize_failed.body':
    'Could not build the new settings.json. Nothing was changed.',
  'error.install_failed.reason.state_failed.body':
    'Could not save the installation state. settings.json was not changed.',
  'error.install_failed.reason.concurrent_modification.body':
    'settings.json changed during the operation, so the monitor left it alone. Try again.',
  'error.install_failed.reason.write_failed.body':
    'Could not write settings.json. The file was not changed; check that it is not locked and try again.',
  'error.install_failed.reason.shim_missing.body':
    'The command executable was not found next to the monitor. Reinstall the monitor.',
  'error.install_failed.reason.copy_failed.body':
    'Could not copy the command executable to the data folder. settings.json was not changed.',
  'error.install_failed.reason.backup_failed.body':
    'Could not save the settings.json backup, so it was not changed.',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': 'Could not remove the command',
  'error.install_failed.reason.foreign_status_line.body':
    "The configured status line is not the monitor's (maybe a hand-edited copy), so nothing was removed.",

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body':
    'The configured CA file was not found. Choose the right file in settings.',
  'error.tls_error.reason.ca_too_large.body':
    'The CA file is too large to read. Choose a smaller .pem file in settings.',
  'error.tls_error.reason.ca_unreadable.body':
    'Could not read the CA file. Check the path and permissions in settings.',
  'error.tls_error.reason.ca_invalid.body':
    'The CA file does not contain a valid certificate. Choose a valid .pem file in settings.',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    'The server response is not in the expected format. Check that the URL points to LiteLLM.',
  'error.endpoint_changed.reason.no_record.body':
    'The response has no key, user or team record. Check the URL and the scope.',
  'error.endpoint_changed.reason.id_missing.body':
    'The response does not confirm the ID that was queried. Check the user or team ID.',
  'error.endpoint_changed.reason.id_mismatch.body':
    'The response is for a different ID than the one provided. Check the user or team ID.',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': 'Local server not ready yet',
  'error.not_running.reason.no_port.body':
    'Antigravity is open, but its local server does not offer an address yet. This usually passes in a moment: try again.',
  'error.unauthorized.reason.no_token.title': 'Local token unavailable',
  'error.unauthorized.reason.no_token.body':
    'The monitor could not read the session token of the IDE local server. Restart Antigravity and try again.',

  'error.redirect':
    'The server tried to redirect the request, and the monitor does not follow redirects.',
  'error.retry_after': 'Wait requested by the source: {wait}.',
  'error.field_hint': 'Field with a problem: {field}.',
  'field.value': 'API key',
  'field.baseUrl': 'server URL',
  'field.entityId': 'user or team ID',
  'field.authHeader': 'authentication header',
  'field.pemPath': 'CA file (.pem)',
  'field.host': 'server without certificate verification',
  'field.configDir': 'Claude Code configuration folder',
  'field.binaryPath': 'Codex executable path',
  'field.dataDir': 'OpenCode data folder',
  'field.shimConsent': 'consent to install the command',
  'field.limit': 'number of past windows',
} as const satisfies Record<string, string>;
