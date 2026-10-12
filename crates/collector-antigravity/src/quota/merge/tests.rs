use super::*;

const FIVE: Option<Cadence> = Some(Cadence::FiveHour);
const WEEK: Option<Cadence> = Some(Cadence::Weekly);

fn win(pool: &Pool, cadence: Option<Cadence>, used: Option<f64>) -> PoolWindow {
    let mut window = PoolWindow::new(pool.clone());
    window.cadence = cadence;
    window.used_pct = used;
    window
}

fn read(plan: Option<&str>, windows: Vec<PoolWindow>) -> Read {
    Read { plan: plan.map(str::to_owned), windows }
}

type Shape = (String, Option<Cadence>, Option<f64>);

fn shape(read: &Read) -> Vec<Shape> {
    read.windows.iter().map(|w| (w.pool.id().to_owned(), w.cadence, w.used_pct)).collect()
}

fn row(pool: &str, cadence: Option<Cadence>, used: Option<f64>) -> Shape {
    (pool.to_owned(), cadence, used)
}

#[test]
fn the_summary_wins_where_it_has_a_share() {
    let status = read(Some("Pro"), vec![win(&Pool::Gemini, FIVE, Some(80.0))]);
    let summary = read(
        None,
        vec![win(&Pool::Gemini, WEEK, Some(10.0)), win(&Pool::Gemini, FIVE, Some(20.0))],
    );
    let merged = merge(Some(status), Some(summary));
    assert_eq!(shape(&merged), [row("gemini", FIVE, Some(20.0)), row("gemini", WEEK, Some(10.0)),]);
    assert_eq!(merged.plan.as_deref(), Some("Pro"));
}

#[test]
fn a_pool_the_summary_lacks_comes_from_the_status() {
    let status = read(
        None,
        vec![win(&Pool::Gemini, FIVE, Some(80.0)), win(&Pool::ClaudeGpt, FIVE, Some(30.0))],
    );
    let summary = read(None, vec![win(&Pool::Gemini, WEEK, Some(10.0))]);
    let merged = merge(Some(status), Some(summary));
    assert_eq!(
        shape(&merged),
        [row("gemini", WEEK, Some(10.0)), row("claude_gpt", FIVE, Some(30.0)),]
    );
}

#[test]
fn a_share_beats_a_pool_the_summary_could_not_read() {
    let status = read(None, vec![win(&Pool::Gemini, FIVE, Some(30.0))]);
    let summary = read(None, vec![win(&Pool::Gemini, FIVE, None), win(&Pool::Gemini, WEEK, None)]);
    let merged = merge(Some(status), Some(summary));
    assert_eq!(shape(&merged), [row("gemini", FIVE, Some(30.0))]);
}

#[test]
fn when_nobody_has_a_share_the_summary_keeps_its_stated_cadences() {
    let status = read(None, vec![win(&Pool::Gemini, FIVE, None)]);
    let summary = read(None, vec![win(&Pool::Gemini, FIVE, None), win(&Pool::Gemini, WEEK, None)]);
    let merged = merge(Some(status.clone()), Some(summary));
    assert_eq!(shape(&merged), [row("gemini", FIVE, None), row("gemini", WEEK, None)]);
    // Without a summary there is only the status to show.
    assert_eq!(shape(&merge(Some(status), None)), [row("gemini", FIVE, None)]);
}

#[test]
fn pools_come_in_a_fixed_order_and_the_short_window_first() {
    let other = |name: &str| Pool::Other(name.to_owned());
    let summary = read(
        None,
        vec![
            win(&other("zeta"), WEEK, Some(1.0)),
            win(&Pool::ClaudeGpt, WEEK, Some(2.0)),
            win(&other("alpha"), None, Some(3.0)),
            win(&Pool::ClaudeGpt, FIVE, Some(4.0)),
            win(&Pool::Gemini, None, Some(5.0)),
            win(&Pool::Gemini, WEEK, Some(6.0)),
            win(&Pool::Gemini, FIVE, Some(7.0)),
        ],
    );
    assert_eq!(
        shape(&merge(None, Some(summary))),
        [
            row("gemini", FIVE, Some(7.0)),
            row("gemini", WEEK, Some(6.0)),
            row("gemini", None, Some(5.0)),
            row("claude_gpt", FIVE, Some(4.0)),
            row("claude_gpt", WEEK, Some(2.0)),
            row("alpha", None, Some(3.0)),
            row("zeta", WEEK, Some(1.0)),
        ]
    );
}

#[test]
fn a_cadence_listed_twice_keeps_the_most_used_and_never_an_unknown() {
    let summary = read(
        None,
        vec![
            win(&Pool::Gemini, FIVE, Some(20.0)),
            win(&Pool::Gemini, FIVE, Some(60.0)),
            win(&Pool::Gemini, FIVE, None),
            win(&Pool::Gemini, WEEK, None),
            win(&Pool::Gemini, WEEK, Some(5.0)),
        ],
    );
    assert_eq!(
        shape(&merge(None, Some(summary))),
        [row("gemini", FIVE, Some(60.0)), row("gemini", WEEK, Some(5.0)),]
    );
}

#[test]
fn windows_without_a_cadence_are_all_kept() {
    let summary =
        read(None, vec![win(&Pool::Gemini, None, Some(1.0)), win(&Pool::Gemini, None, Some(2.0))]);
    assert_eq!(merge(None, Some(summary)).windows.len(), 2);
}

#[test]
fn the_plan_falls_back_to_the_summary_and_nothing_is_nothing() {
    let summary = read(Some("Ultra"), vec![win(&Pool::Gemini, FIVE, Some(1.0))]);
    assert_eq!(merge(None, Some(summary.clone())).plan.as_deref(), Some("Ultra"));
    let status = read(Some("Pro"), vec![]);
    assert_eq!(merge(Some(status), Some(summary)).plan.as_deref(), Some("Pro"));
    assert_eq!(merge(None, None), read(None, vec![]));
}

#[test]
fn the_list_is_bounded() {
    let many: Vec<PoolWindow> = (0..100)
        .map(|index| win(&Pool::Other(format!("pool_{index:03}")), WEEK, Some(1.0)))
        .collect();
    assert_eq!(merge(None, Some(read(None, many))).windows.len(), MAX_WINDOWS);
}
