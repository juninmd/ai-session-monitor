// What the params of a failure add to its message: the `reason` a collector gives (closed
// vocabularies, worded per error code), the sentences for a redirect and a wait, and the names
// of the settings fields. A reason with a `.title` replaces the title of the code as well.
export const errorDetailMessages = {
  'error.install_failed.reason.status_line_too_large.body':
    '你当前的状态栏命令过大，监视器无法将其串联。未做任何更改。',
  'error.install_failed.reason.path_unsupported.body':
    '数据文件夹路径含有状态栏命令不接受的字符。未做任何更改。',
  'error.install_failed.reason.serialize_failed.body': '无法生成新的 settings.json。未做任何更改。',
  'error.install_failed.reason.state_failed.body': '无法保存安装状态。settings.json 未被更改。',
  'error.install_failed.reason.concurrent_modification.body':
    'settings.json 在操作期间被修改，因此监视器没有改动它。请重试。',
  'error.install_failed.reason.write_failed.body':
    '无法写入 settings.json。文件未被更改；请检查它是否被锁定，然后重试。',
  'error.install_failed.reason.shim_missing.body':
    '在监视器旁找不到该命令的可执行文件。请重新安装监视器。',
  'error.install_failed.reason.copy_failed.body':
    '无法将命令的可执行文件复制到数据文件夹。settings.json 未被更改。',
  'error.install_failed.reason.backup_failed.body': '无法保存 settings.json 的备份，因此未做更改。',
  // Only a removal is refused for this reason.
  'error.install_failed.reason.foreign_status_line.title': '无法移除该命令',
  'error.install_failed.reason.foreign_status_line.body':
    '当前配置的状态栏不是监视器安装的（可能是手动编辑过的副本），因此未移除任何内容。',

  // LiteLLM: the CA file the user chose, not the certificate of the server.
  'error.tls_error.reason.ca_missing.body': '找不到配置的 CA 文件。请在设置中选择正确的文件。',
  'error.tls_error.reason.ca_too_large.body':
    'CA 文件太大，无法读取。请在设置中选择较小的 .pem 文件。',
  'error.tls_error.reason.ca_unreadable.body': '无法读取 CA 文件。请在设置中检查路径和权限。',
  'error.tls_error.reason.ca_invalid.body':
    'CA 文件不包含有效的证书。请在设置中选择有效的 .pem 文件。',

  // LiteLLM: which part of the answer did not look like a key, user or team record.
  'error.endpoint_changed.reason.not_an_object.body':
    '服务器响应不是预期的格式。请确认 URL 指向 LiteLLM。',
  'error.endpoint_changed.reason.no_record.body':
    '响应中没有密钥、用户或团队记录。请检查 URL 和范围。',
  'error.endpoint_changed.reason.id_missing.body': '响应未确认所查询的 ID。请检查用户或团队 ID。',
  'error.endpoint_changed.reason.id_mismatch.body':
    '响应对应的 ID 与提供的不一致。请检查用户或团队 ID。',

  // Antigravity: the IDE's local server is there, but there is nothing to call.
  'error.not_running.reason.no_port.title': '本地服务尚未就绪',
  'error.not_running.reason.no_port.body':
    'Antigravity 已打开，但其本地服务还没有提供地址。通常稍等片刻即可，请重试。',
  'error.unauthorized.reason.no_token.title': '本地令牌不可用',
  'error.unauthorized.reason.no_token.body':
    '监视器无法读取 IDE 本地服务的会话令牌。请重启 Antigravity 后重试。',

  // Cursor: the local database (state.vscdb) and the Copilot extension's sign-in files.
  'error.timeout.reason.locked.body': 'Cursor 的本地数据库正被占用，请稍后重试。',
  'error.timeout.reason.timeout.body': '读取 Cursor 本地数据库耗时过长，请稍后重试。',
  'error.endpoint_changed.reason.schema.body':
    'Cursor 本地数据库的结构与监视器预期不同。监视器可能需要更新。',
  'error.settings_unreadable.reason.permission.body':
    '没有权限打开 Cursor 的本地数据库。请检查该文件的权限。',
  'error.settings_unreadable.reason.not_a_database.body': '找到的 Cursor 文件不是数据库。',
  'error.settings_unreadable.reason.other.body': '无法读取 Cursor 的本地数据库。',
  'error.not_signed_in.reason.not_found.body':
    '在 Copilot 扩展的文件夹中找不到 GitHub 令牌。请在编辑器中登录 Copilot，或在设置中粘贴令牌。',
  'error.not_signed_in.reason.no_token.body':
    'Copilot 扩展的文件中没有 github.com 的令牌。请在编辑器中登录 Copilot，或在设置中粘贴令牌。',
  'error.not_signed_in.reason.unreadable.body':
    '无法读取 Copilot 扩展的登录文件。请检查其文件夹的权限。',

  'error.redirect': '服务器尝试重定向请求，而监视器不会跟随重定向。',
  'error.retry_after': '数据源要求等待：{wait}。',
  'error.field_hint': '有问题的字段：{field}。',
  'field.value': 'API 密钥',
  'field.baseUrl': '服务器 URL',
  'field.entityId': '用户或团队 ID',
  'field.authHeader': '认证请求头',
  'field.pemPath': 'CA 文件（.pem）',
  'field.host': '不验证证书的服务器',
  'field.configDir': 'Claude Code 配置文件夹',
  'field.binaryPath': 'Codex 可执行文件路径',
  'field.dataDir': 'OpenCode 数据文件夹',
  'field.shimConsent': '同意安装该命令',
  'field.limit': '历史窗口数量',
} as const satisfies Record<string, string>;
