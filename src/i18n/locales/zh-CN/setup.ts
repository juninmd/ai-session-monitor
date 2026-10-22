// First-run connection surfaces: consent cards, secret fields, OpenCode and LiteLLM forms.
export const setupMessages = {
  'consent.skip': '以后再说',
  'consent.details': '将读取哪些内容',

  'claude.connect.title': '连接 Claude Code',
  'claude.connect.lead':
    '监视器会在 Claude Code 状态栏中添加一个小命令，仅把额度百分比（5 小时和本周）存入本地文件，不会读取任何登录信息。',
  'claude.connect.detail':
    '会修改 Claude Code 的 settings.json（并先备份），同时串联你当前的状态栏。仅把额度写入 %LOCALAPPDATA%，不联网，也不涉及令牌。适用于 Pro 和 Max 账户。数字在会话收到第一条回复后才会出现，且仅在 Claude Code 打开时更新。',
  'claude.connect.allow': '安装命令',
  'claude.connect.tip':
    '提示：Windows 会把新图标收进溢出箭头菜单。将监视器图标拖到任务栏上，即可一直显示。',

  'codex.connect.title': '连接 Codex',
  'codex.connect.lead':
    '监视器会启动本机安装的 codex app-server，查询 5 小时窗口和本周的剩余额度。',
  'codex.connect.detail':
    '在你的电脑上以只读方式运行 codex 程序。数据不会离开这台电脑，应用也不会触碰 Codex 的登录文件。OpenAI 将 app-server 标记为实验性功能，因此格式可能随版本变化。',
  'codex.connect.allow': '允许读取',

  'antigravity.connect.title': '连接 Antigravity',
  'antigravity.connect.lead':
    '额度来自已安装的 agy（CLI）或 IDE 的本地服务。这是非官方途径，Antigravity 更新后可能失效。',
  'antigravity.connect.detail':
    '经你许可，监视器可以运行已安装的 agy，仅用于读取 /usage（不消耗 Token，也不读取登录数据），并且/或者读取 IDE 服务的命令行以获取本地会话令牌，仅与 127.0.0.1 通信。不会写入磁盘，也不会发送到互联网。',
  'antigravity.connect.allow': '允许本地读取',

  'cursor.connect.title': '连接 Cursor',
  'cursor.connect.lead':
    '经你许可，监视器会读取 Cursor 保存在本地数据库中的登录令牌，并用它向 cursor.com 查询你的套餐已用多少。这是非官方途径，Cursor 更新后可能失效。',
  'cursor.connect.detail':
    '以只读方式打开 Cursor 的本地数据库（state.vscdb），仅读取一条记录：登录令牌。令牌只会像 Cursor 控制台那样发送到 cursor.com，不会被保存或显示。不会读取其他任何记录（聊天、文件、电子邮件），也不会写入任何内容。Cursor 没有公开套餐用量的 API，因此首次使用时请将数字与控制台对照。',
  'cursor.connect.allow': '允许读取',
  'cursor.signin.title': '登录 Cursor',
  'cursor.signin.body':
    '监视器找不到有效的登录令牌。请打开 Cursor 并登录（打开它也会刷新已过期的令牌），然后重新读取。',
  'cursor.dashboard': '打开 Cursor 控制台',
  'cursor.note':
    '非官方读取 Cursor 用量摘要，使用 Cursor 保存在这台电脑上的登录令牌。请与控制台对照。套餐包含的用量只以百分比显示；无限量套餐没有窗口。',

  'shim.title': '状态栏命令',
  'shim.state.not_installed': '未安装',
  'shim.state.installed': '已安装',
  'shim.state.installed_chained': '已安装，并与你的状态栏串联',
  'shim.state.foreign': '正在使用其他状态栏',
  'shim.state.outdated': '已过期：找不到可执行文件',
  'shim.state.unreadable': '无法读取 settings.json',
  'shim.hint.foreign': '安装后，你当前的状态栏仍可正常使用：监视器会将其串联。',
  'shim.hint.unreadable': '监视器不会改动无法读取的文件。请修复 settings.json 后再检查。',
  'shim.path': '文件：{path}',
  'shim.install': '安装命令',
  'shim.reinstall': '重新安装命令',
  'shim.uninstall': '移除命令',
  'shim.recheck': '重新检查',
  'shim.loading': '正在检查 Claude Code',

  'secret.saved': '密钥已保存',
  'secret.not_saved': '未保存密钥',
  'secret.placeholder': '粘贴密钥',
  'secret.placeholder_keep': '留空则保留已保存的密钥',
  'secret.hint': '密钥保存在 Windows 凭据管理器中，不会出现在界面里。',
  'secret.save': '保存密钥',
  'secret.clear': '移除密钥',
  'secret.empty': '请粘贴 API 密钥。',
  'secret.saved_ok': '密钥已保存。',
  'secret.cleared_ok': '密钥已移除。',

  'opencode.title': '连接 OpenCode Go',
  'opencode.lead':
    '粘贴你的 OpenCode Go API 密钥，即可读取真实的窗口（5 小时、本周和本月）。没有密钥时，只能看到本地用量估算。',
  'opencode.key_label': 'OpenCode Go API 密钥',
  'opencode.console': '打开 OpenCode Go 控制台',
  'opencode.billing': '查看 Zen 账单',
  'opencode.zen_note': 'Zen（预付费）余额没有 API：请查看账单页面。',
  'opencode.local_label': '本地估算（仅限这台电脑）',
  'opencode.local_hint':
    '以只读方式打开 OpenCode 数据库，仅按标价汇总费用和 Token。这不是真实额度。',
  'opencode.data_dir': 'OpenCode 数据文件夹',
  'opencode.data_dir_hint': '留空则使用默认文件夹。',
  'opencode.save_settings': '保存文件夹',

  'kimi.title': '连接 Kimi Code',
  'kimi.lead':
    '粘贴你的 Kimi Code API 密钥，即可读取套餐额度：5 小时窗口、本周和本月总量（套餐有提供时）。',
  'kimi.detail':
    '非官方读取：用量 API 没有公开文档，可能随时变更。密钥只会发送到 api.kimi.com，并保存在 Windows 凭据管理器中。不使用其他工具的 Cookie 或登录信息。Moonshot 开放平台的密钥属于另一个产品，在这里无法使用。',
  'kimi.key_label': 'Kimi Code API 密钥',
  'kimi.console': '打开 Kimi Code 控制台',
  'kimi.note': '非官方读取 Kimi Code 用量 API。本周和本月仅在套餐提供时才会显示。',

  'zai.title': '连接 z.ai',
  'zai.lead': '粘贴你的 GLM Coding 套餐 API 密钥，即可读取 5 小时窗口、本周和每月 MCP 工具额度。',
  'zai.detail':
    '非官方读取：额度 API 没有公开文档，可能随时变更。密钥只会发送到 api.z.ai，并保存在 Windows 凭据管理器中。不使用其他工具的 Cookie 或登录信息。不会读取 BigModel 账户（open.bigmodel.cn）。',
  'zai.key_label': 'z.ai API 密钥',
  'zai.console': '打开 z.ai 密钥页',
  'zai.note': '非官方读取 GLM Coding 套餐额度 API，使用国际站服务器（api.z.ai）。',

  'openrouter.title': '连接 OpenRouter',
  'openrouter.lead':
    '粘贴你的 OpenRouter API 密钥，即可查看密钥的支出上限、用量，以及（密钥允许时）积分余额。',
  'openrouter.detail':
    '官方读取，通过 OpenRouter 公开文档的 API。密钥只会发送到 openrouter.ai，并保存在 Windows 凭据管理器中。积分余额需要管理密钥；使用普通密钥时，只会显示密钥上限和用量。',
  'openrouter.key_label': 'OpenRouter API 密钥',
  'openrouter.console': '打开 OpenRouter 密钥页',
  'openrouter.note':
    '官方读取 OpenRouter API。密钥没有设置上限时不显示百分比，只显示用量。余额仅在使用管理密钥时显示。',

  'deepseek.title': '连接 DeepSeek',
  'deepseek.lead': '粘贴你的 DeepSeek API 密钥，即可查看账户余额（按各币种分别显示）。',
  'deepseek.detail':
    '官方读取，通过 DeepSeek 公开文档的 API。密钥只会发送到 api.deepseek.com，并保存在 Windows 凭据管理器中。余额没有上限或重置时间，因此没有百分比；人民币和美元不会相加。',
  'deepseek.key_label': 'DeepSeek API 密钥',
  'deepseek.console': '打开 DeepSeek 密钥页',
  'deepseek.note': '官方读取 DeepSeek 余额 API。这是金额而不是额度：没有百分比，也没有重置。',

  'deepinfra.title': '连接 DeepInfra',
  'deepinfra.lead': '粘贴你的 DeepInfra API 密钥，即可查看余额、支出上限和本月支出。',
  'deepinfra.detail':
    '官方读取，通过 DeepInfra 账单 API。密钥只会发送到 api.deepinfra.com，并保存在 Windows 凭据管理器中。没有支出上限时不显示百分比，只显示美元金额。',
  'deepinfra.key_label': 'DeepInfra API 密钥',
  'deepinfra.console': '打开 DeepInfra 密钥页',
  'deepinfra.note': '官方读取 DeepInfra 账单 API。数值按美元读取；没有支出上限时不显示百分比。',

  'moonshot.title': '连接 Moonshot',
  'moonshot.lead': '粘贴你的 Moonshot 平台（platform.moonshot.ai）API 密钥，即可查看账户余额。',
  'moonshot.detail':
    '官方读取，通过 Moonshot 公开文档的余额 API。密钥只会发送到 api.moonshot.ai，并保存在 Windows 凭据管理器中。kimi.com 平台的密钥是独立的，在这里无法使用。余额是金额：没有百分比，也没有重置。',
  'moonshot.key_label': 'Moonshot API 密钥',
  'moonshot.console': '打开 Moonshot 密钥页',
  'moonshot.note': '官方读取 Moonshot 余额 API，单位为美元。有欠款时，现金余额可能为负数。',

  'novita.title': '连接 Novita',
  'novita.lead': '粘贴你的 Novita API 密钥，即可查看账户余额、信用额度和待结算费用。',
  'novita.detail':
    '官方读取，通过 Novita 公开文档的账单 API。密钥只会发送到 api.novita.ai，并保存在 Windows 凭据管理器中。余额是金额：没有百分比，也没有重置。',
  'novita.key_label': 'Novita API 密钥',
  'novita.console': '打开 Novita 密钥页',
  'novita.note': '官方读取 Novita 余额 API，单位为美元。有欠款时，余额可能为负数。',

  'ollama.title': '连接 Ollama Cloud',
  'ollama.lead': '粘贴 Ollama Cloud API 密钥，即可查看套餐包含的积分和已购余额。',
  'ollama.detail':
    '官方读取，通过 Ollama Cloud 公开文档的余额 API。请在 ollama.com/settings/keys 创建密钥：绝不会读取 CLI 的订阅密钥。密钥只会发送到 ollama.com，并保存在 Windows 凭据管理器中。响应格式取决于套餐（每月积分，或会话和每周窗口）。',
  'ollama.key_label': 'Ollama Cloud API 密钥',
  'ollama.console': '打开 Ollama Cloud 密钥页',
  'ollama.note':
    '官方读取 Ollama Cloud 余额 API。当前套餐显示美元积分；旧套餐显示会话和每周百分比。',

  'minimax.title': '连接 MiniMax',
  'minimax.lead': '粘贴你的 MiniMax Token Plan 密钥，即可查看各模型的额度：当前时段和本周。',
  'minimax.detail':
    '非官方读取：Token Plan 额度接口没有公开的数据结构，可能随时变更。密钥只会发送到 api.minimax.io（国际站服务器），并保存在 Windows 凭据管理器中。首次使用时，请将数字与 MiniMax 控制台对照。',
  'minimax.key_label': 'MiniMax Token Plan 密钥',
  'minimax.console': '打开 MiniMax 密钥页',
  'minimax.note': '非官方读取 MiniMax Token Plan 额度，按模型显示。套餐中没有额度的模型不会显示。',

  'copilot.title': '连接 GitHub Copilot',
  'copilot.lead':
    '粘贴 GitHub 令牌，或让监视器读取 Copilot 扩展保存在磁盘上的令牌，即可查看本月已用的高级请求。这是非官方途径。',
  'copilot.detail':
    '非官方读取：使用 Copilot 编辑器为显示自身额度而调用的 GitHub 内部接口。该接口没有公开文档，可能随时变更，恕不另行通知。粘贴的令牌保存在 Windows 凭据管理器中。经你许可，监视器也可以改为读取 Copilot 扩展保存在磁盘上的 GitHub 令牌（其自身文件夹中的两个固定文件，只读）。两种令牌都只会发送到 api.github.com。不使用 GitHub Enterprise 令牌。首次使用时，请将数字与 github.com/settings/copilot 对照。',
  'copilot.key_label': 'GitHub 令牌（可选）',
  'copilot.console': '打开 GitHub Copilot 设置',
  'copilot.local_label': '读取 Copilot 扩展令牌（仅限这台电脑）',
  'copilot.local_hint':
    '非官方：读取 Copilot 扩展保存在磁盘上的 GitHub 令牌（两个固定文件，只读），并且只发送到 api.github.com。粘贴令牌时无需开启。',
  'copilot.note':
    '非官方读取 GitHub Copilot 额度。重置日期可能是估算值（每月 1 日，UTC）。无限量的额度作为详情列出，不显示百分比。',

  'litellm.title': '连接 LiteLLM',
  'litellm.title_edit': 'LiteLLM 连接',
  'litellm.url': '服务器 URL',
  'litellm.url_placeholder': 'https://litellm.yourcompany.com',
  'litellm.url_hint': '必须使用 HTTPS。仅 localhost 接受 HTTP。',
  'litellm.url_required': '请输入服务器 URL。',
  'litellm.url_invalid': '请输入完整的 URL，例如 https://litellm.yourcompany.com。',
  'litellm.url_http': '请使用 https://。仅 localhost 可使用 HTTP。',
  'litellm.url_parts': '请从 URL 中去掉用户名、密码、查询参数和 #。',
  'litellm.key': 'API 密钥',
  'litellm.key_required': '请粘贴 LiteLLM API 密钥。',
  'litellm.key_rebind': '已保存的密钥只适用于当前连接。连接已变更，请重新粘贴密钥。',
  'litellm.scope': '监测对象',
  'litellm.scope.key': '此密钥',
  'litellm.scope.user': '用户',
  'litellm.scope.team': '团队',
  'litellm.entity.user': '用户 ID',
  'litellm.entity.team': '团队 ID',
  'litellm.entity_required': '请输入 ID。',
  'litellm.advanced': '高级',
  'litellm.auth_header': '认证请求头',
  'litellm.auth_header_hint': '仅当你的代理使用自定义请求头传递密钥时才需要修改。',
  'litellm.auth_header_invalid': '只能使用字母、数字和连字符。',
  'litellm.poll': '刷新间隔',
  'litellm.timeout': '超时',
  'litellm.tls': 'TLS 证书',
  'litellm.tls.system': '使用 Windows 证书',
  'litellm.tls.custom_ca': '自定义 CA',
  'litellm.tls.insecure': '不验证',
  'litellm.pem': 'CA 文件（.pem）',
  'litellm.pem_placeholder': 'C:\\certs\\ca.pem',
  'litellm.pem_required': '请输入 .pem 文件的路径。',
  'litellm.insecure_host': '不验证的服务器',
  'litellm.insecure_host_hint': '仅对这台服务器跳过证书验证。',
  'litellm.insecure_host_required': '请输入服务器。',
  'litellm.insecure_warning': '连接将不会被验证，密钥可能被截获。请仅在可信网络中使用。',
  'litellm.insecure_confirm': '我了解风险，并希望忽略此服务器的证书。',
  'litellm.insecure_confirm_required': '请确认后再继续，将不验证证书。',
  'litellm.save': '保存并连接',
  'litellm.saving': '正在保存…',
  'litellm.saved': '设置已保存，正在读取服务器…',
  'litellm.edit': '编辑连接',
  'litellm.delay_note':
    '服务器上的支出和重置最多可能延迟约 10 分钟。上限可能设在用户或团队上，而不仅是密钥。',
} as const satisfies Record<string, string>;
