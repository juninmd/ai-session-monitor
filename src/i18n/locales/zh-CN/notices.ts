// Notice codes from docs/ipc.md. An unknown code renders `notice.unknown` with the code in it,
// so a missing translation is visible instead of silent.
export const noticeMessages = {
  'notice.code.claude_waiting_first_session':
    '命令已安装。在 Claude Code 会话中收到第一条回复后才会有额度数据。',
  'notice.code.claude_no_rate_limits':
    'Claude Code 未报告额度（按 API 密钥计费，或套餐没有额度限制）。',
  'notice.code.claude_shim_outdated':
    '在 Claude Code 状态栏中找不到已安装的命令。请重新安装以恢复读取。',
  'notice.code.claude_oauth_partial':
    '已开启按模型用量（claude /usage 或账户用量 API），但读取失败（{reason}）。仅显示状态栏的数据。',
  'notice.code.codex_rollout_fallback':
    'codex app-server 没有响应（{reason}）。正在读取最近一次记录的会话，数字可能已过时。',
  'notice.code.codex_no_limits': 'Codex 账户未报告额度（使用 API 密钥登录）。',
  'notice.code.antigravity_pool_unknown': '{pool}额度没有返回数据，显示为未知，而不是零。',
  'notice.code.litellm_no_budget': '此密钥、用户或团队没有预算上限。',
  'notice.code.litellm_level_unreadable':
    '无法读取{level}预算（{reason}）。那里可能有上限，但此处看不到。',
  'notice.code.litellm_windows_truncated':
    '预算窗口太多，无法全部显示。已省略用量最少的：{dropped}。',
  'notice.code.opencode_window_idle': '{window}窗口为 0%：要等到下一次请求才开始计量。',
  'notice.code.opencode_rate_limited': '{window}窗口已被限流。',
  'notice.code.opencode_no_key': '仅显示本地估算。粘贴 Go API 密钥即可查看真实窗口。',
  'notice.code.opencode_local_estimate': '本地数值按标价估算，仅来自这台电脑，并不是实际额度。',
  'notice.code.opencode_db_unreadable': '无法读取本地 OpenCode 数据库：{reason}。',
  'notice.unknown': '未翻译的通知：{code}',
  'notice.action.reinstall': '重新安装',
  'notice.action.settings': '输入密钥',
  'notice.param.window.rolling': '5 小时',
  'notice.param.window.weekly': '每周',
  'notice.param.window.monthly': '每月',
  'notice.param.level.key': '密钥',
  'notice.param.level.user': '用户',
  'notice.param.level.team': '团队',
  'notice.param.dropped.other': '{n} 个窗口',
  // The OpenCode database reasons are their own vocabulary: `not_found` there is a missing file.
  'notice.reason.opencode_db_unreadable.invalid_path': '数据文件夹路径无效',
  'notice.reason.opencode_db_unreadable.not_found': '找不到数据库文件',
  'notice.reason.opencode_db_unreadable.permission': '没有权限打开数据库文件',
  'notice.reason.opencode_db_unreadable.locked': '数据库正被其他程序占用',
  'notice.reason.opencode_db_unreadable.not_a_database': '找到的文件不是数据库',
  'notice.reason.opencode_db_unreadable.schema': '数据库格式已变更',
  'notice.reason.opencode_db_unreadable.timeout': '读取数据库超时',
  'notice.reason.opencode_db_unreadable.other': '原因不明',
  'notice.severity.info': '信息',
  'notice.severity.warn': '警告',
  'notice.severity.error': '错误',
} as const satisfies Record<string, string>;
