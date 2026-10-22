//! Sample views behind the snapshot golden: one per harness, covering every state and window shape.

use std::collections::BTreeMap;

use usage_model::view::WindowKind;
use usage_model::{
    Amount, AmountUnit, ErrorCode, Fact, HarnessError, HarnessId, HarnessView, Notice, Severity,
    Source, UsageWindow, ViewState,
};

use super::NOW;

fn window(
    kind: WindowKind,
    id: &str,
    minutes: Option<u32>,
    used: f64,
    left_min: i64,
) -> UsageWindow {
    let mut w = UsageWindow::new(id, kind);
    w.window_minutes = minutes;
    w.used_pct = Some(used);
    w.resets_at = Some(NOW + left_min * 60);
    w
}

fn ready(id: HarnessId, source: &str, official: bool) -> HarnessView {
    let mut view = HarnessView::with_state(id, ViewState::Ready);
    view.source = Some(Source { id: source.into(), official });
    view.read_at = Some(NOW - 30);
    view
}

fn currency_fact(id: &str, number: f64, unit: &str) -> Fact {
    Fact { id: id.into(), number: Some(number), text: None, unit: Some(unit.into()) }
}

fn usd_fact(id: &str, number: f64) -> Fact {
    currency_fact(id, number, "usd")
}

pub fn sample_views() -> Vec<HarnessView> {
    let mut claude = ready(HarnessId::Claude, "statusline", true);
    claude.plan = Some("Max 5x".into());
    claude.windows = vec![
        window(WindowKind::FiveHour, "five_hour", Some(300), 48.0, 113),
        window(WindowKind::Weekly, "weekly", Some(10_080), 45.0, 5_980),
    ];

    let mut antigravity = ready(HarnessId::Antigravity, "language_server", false);
    let mut gemini = window(WindowKind::FiveHour, "gemini:five_hour", Some(300), 54.0, 132);
    gemini.group = Some("gemini".into());
    gemini.label = Some("Gemini Pro / Flash".into());
    let mut claude_gpt = window(WindowKind::Weekly, "claude_gpt:weekly", Some(10_080), 20.0, 4_000);
    claude_gpt.group = Some("claude_gpt".into());
    antigravity.windows = vec![gemini, claude_gpt];
    antigravity.stale = true;
    antigravity.error =
        Some(HarnessError::new(ErrorCode::RateLimited, true).with_param("retryAfterSecs", "120"));
    antigravity.next_attempt_at = Some(1_800_000_300);

    let mut opencode = ready(HarnessId::Opencode, "go_usage", false);
    opencode.windows = vec![
        window(WindowKind::FiveHour, "rolling", Some(300), 12.0, 250),
        window(WindowKind::Weekly, "weekly", Some(10_080), 34.0, 3_000),
        {
            let mut monthly = window(WindowKind::Monthly, "monthly", None, 100.0, 9_000);
            monthly.used_pct = Some(100.0);
            monthly
        },
    ];
    opencode.facts = vec![
        Fact {
            id: "local_cost_5h".into(),
            number: Some(0.0037),
            text: None,
            unit: Some("usd".into()),
        },
        Fact { id: "plan_note".into(), number: None, text: Some("Go".into()), unit: None },
    ];
    opencode.notices = vec![Notice {
        code: "opencode_window_idle".into(),
        severity: Severity::Info,
        params: BTreeMap::new(),
    }];

    let mut kimi = ready(HarnessId::Kimi, "coding_usages", false);
    kimi.plan = Some("Intermediate".into());
    kimi.windows = vec![
        {
            let mut five = window(WindowKind::FiveHour, "five_hour", Some(300), 25.0, 120);
            five.amount =
                Some(Amount { used: 50.0, limit: Some(200.0), unit: AmountUnit::Requests });
            five
        },
        window(WindowKind::Weekly, "weekly", None, 25.0, 4_320),
        window(WindowKind::Monthly, "monthly", None, 40.0, 20_000),
    ];

    let mut zai = ready(HarnessId::Zai, "quota_limit", false);
    zai.plan = Some("Pro".into());
    zai.windows = vec![
        window(WindowKind::FiveHour, "five_hour", Some(300), 12.0, 120),
        window(WindowKind::Weekly, "weekly", Some(10_080), 40.0, 4_320),
        window(WindowKind::Monthly, "mcp", None, 25.0, 20_000),
    ];

    let mut openrouter = ready(HarnessId::Openrouter, "key_credits", true);
    let mut key_limit = UsageWindow::new("key_limit", WindowKind::Budget);
    key_limit.used_pct = Some(25.0);
    key_limit.amount = Some(Amount { used: 2.5, limit: Some(10.0), unit: AmountUnit::Usd });
    openrouter.windows = vec![key_limit];
    openrouter.facts = vec![usd_fact("balance", 74.75), usd_fact("usage_daily", 0.25)];

    let mut deepseek = ready(HarnessId::Deepseek, "user_balance", true);
    deepseek.facts =
        vec![currency_fact("balance_cny", 110.0, "cny"), currency_fact("balance_usd", 3.5, "usd")];

    let mut deepinfra = ready(HarnessId::Deepinfra, "payment_checklist", true);
    let mut spend_limit = UsageWindow::new("spend_limit", WindowKind::Budget);
    spend_limit.used_pct = Some(25.0);
    spend_limit.amount = Some(Amount { used: 2.5, limit: Some(10.0), unit: AmountUnit::Usd });
    deepinfra.windows = vec![spend_limit];
    deepinfra.facts = vec![usd_fact("balance", 12.5), usd_fact("usage_month", 12.34)];

    let mut moonshot = ready(HarnessId::Moonshot, "users_balance", true);
    moonshot.facts =
        vec![usd_fact("balance", 49.58), usd_fact("voucher", 46.58), usd_fact("cash", -2.5)];

    let mut novita = ready(HarnessId::Novita, "balance_detail", true);
    novita.facts = vec![usd_fact("balance", 100.0), usd_fact("cash", 80.0), usd_fact("owed", 0.0)];

    let mut ollama = ready(HarnessId::Ollama, "balance", true);
    let mut included = window(WindowKind::Monthly, "monthly", Some(43_200), 27.5, 20_000);
    included.amount = Some(Amount { used: 27.5, limit: Some(100.0), unit: AmountUnit::Usd });
    ollama.windows = vec![included];
    ollama.facts = vec![usd_fact("included_balance", 72.5), usd_fact("purchased_balance", 25.0)];

    let mut minimax = ready(HarnessId::Minimax, "token_plan_remains", false);
    let mut interval = window(WindowKind::FiveHour, "minimax_m:interval", Some(300), 25.0, 240);
    interval.group = Some("MiniMax-M*".into());
    let mut weekly = window(WindowKind::Weekly, "minimax_m:weekly", None, 20.0, 5_000);
    weekly.group = Some("MiniMax-M*".into());
    weekly.resets_at = None;
    minimax.windows = vec![interval, weekly];

    let mut litellm = ready(HarnessId::Litellm, "key_info", true);
    let mut budget = window(WindowKind::Budget, "budget", None, 25.0, 20_000);
    budget.label = Some("team-a".into());
    budget.amount = Some(Amount { used: 12.5, limit: Some(50.0), unit: AmountUnit::Usd });
    litellm.windows = vec![budget];
    litellm.meta.insert("host".into(), "gateway.example.dev".into());
    litellm.meta.insert("scope".into(), "key".into());

    let mut cursor = ready(HarnessId::Cursor, "usage_summary", false);
    cursor.plan = Some("Pro".into());
    let mut cycle = window(WindowKind::Monthly, "monthly", Some(43_200), 36.0, 20_000);
    cycle.amount = None;
    let mut on_demand = window(WindowKind::Budget, "on_demand", None, 10.0, 20_000);
    on_demand.amount = Some(Amount { used: 5.0, limit: Some(50.0), unit: AmountUnit::Usd });
    cursor.windows = vec![cycle, on_demand];

    let mut copilot = ready(HarnessId::Copilot, "copilot_internal_user", false);
    copilot.plan = Some("Individual Pro".into());
    let mut premium = window(WindowKind::Monthly, "premium_requests", None, 64.0, 18_000);
    premium.amount = Some(Amount { used: 192.0, limit: Some(300.0), unit: AmountUnit::Requests });
    copilot.windows = vec![premium];
    copilot.facts = vec![Fact {
        id: "chat_unlimited".into(),
        number: None,
        text: Some("unlimited".into()),
        unit: None,
    }];

    let mut unknown_pct = ready(HarnessId::Codex, "app_server", true);
    unknown_pct.windows = vec![UsageWindow::new("five_hour", WindowKind::FiveHour)];

    vec![
        claude,
        HarnessView::needs_consent(HarnessId::Codex),
        cursor,
        copilot,
        antigravity,
        opencode,
        kimi,
        zai,
        openrouter,
        deepseek,
        deepinfra,
        moonshot,
        novita,
        ollama,
        minimax,
        litellm,
        unknown_pct,
        HarnessView::loading(HarnessId::Claude),
        HarnessView::disabled(HarnessId::Antigravity),
        HarnessView::needs_setup(HarnessId::Litellm),
        HarnessView::with_state(HarnessId::Opencode, ViewState::NoData),
        HarnessView::needs_setup(HarnessId::Kimi),
        HarnessView::needs_setup(HarnessId::Deepseek),
        HarnessView::error(
            HarnessId::Zai,
            HarnessError::new(ErrorCode::Unauthorized, false)
                .with_param("httpStatus", "401")
                .with_param("ambiguous", "true"),
        ),
        HarnessView::error(
            HarnessId::Litellm,
            HarnessError::new(ErrorCode::Unauthorized, false).with_param("httpStatus", "401"),
        ),
    ]
}
