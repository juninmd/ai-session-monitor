//! Every statement this crate runs against opencode.db. They are constants: nothing in them comes
//! from settings or database content, and a test pins that they name only `session_message` and
//! use the `data` column solely inside `json_*` calls (a message body is never selected).

pub(crate) const TABLE: &str = "session_message";
pub(crate) const REQUIRED_COLUMNS: [&str; 2] = ["type", "data"];
pub(crate) const SCHEMA_PROBE: &str = "PRAGMA table_info(session_message)";

/// One pass over the last 30 days of assistant messages, split into 5 h / 7 d / 30 d with FILTER.
/// Only JSON numbers are summed: text, null, nested or missing values count as absent, and a
/// malformed row is skipped instead of failing the whole read. Values are summed as REAL because
/// SQLite aborts the statement on an integer SUM overflow, and one odd row must not do that.
pub(crate) const AGGREGATE: &str = "\
SELECT
  COUNT(*) FILTER (WHERE created >= :since_5h),
  SUM(cost) FILTER (WHERE created >= :since_5h),
  SUM(tokens_in) FILTER (WHERE created >= :since_5h),
  SUM(tokens_out) FILTER (WHERE created >= :since_5h),
  SUM(tokens_reasoning) FILTER (WHERE created >= :since_5h),
  COUNT(*) FILTER (WHERE created >= :since_7d),
  SUM(cost) FILTER (WHERE created >= :since_7d),
  SUM(tokens_in) FILTER (WHERE created >= :since_7d),
  SUM(tokens_out) FILTER (WHERE created >= :since_7d),
  SUM(tokens_reasoning) FILTER (WHERE created >= :since_7d),
  COUNT(*),
  SUM(cost)
FROM (
  SELECT
    json_extract(data, '$.time.created') AS created,
    CASE WHEN json_type(data, '$.cost') IN ('integer', 'real')
         THEN CAST(json_extract(data, '$.cost') AS REAL) END AS cost,
    CASE WHEN json_type(data, '$.tokens.input') IN ('integer', 'real')
         THEN CAST(json_extract(data, '$.tokens.input') AS REAL) END AS tokens_in,
    CASE WHEN json_type(data, '$.tokens.output') IN ('integer', 'real')
         THEN CAST(json_extract(data, '$.tokens.output') AS REAL) END AS tokens_out,
    CASE WHEN json_type(data, '$.tokens.reasoning') IN ('integer', 'real')
         THEN CAST(json_extract(data, '$.tokens.reasoning') AS REAL) END AS tokens_reasoning
  FROM session_message
  WHERE type = 'assistant'
    AND CASE WHEN json_valid(data) AND json_type(data, '$.time.created') IN ('integer', 'real')
             THEN json_extract(data, '$.time.created') >= :since_30d END
)";

/// Run only when the last 30 days hold no messages: tells "idle for a month" (old messages still
/// have a readable time) from "the time field moved" (none has one).
pub(crate) const DATED_ROWS_PROBE: &str = "\
SELECT
  COUNT(*),
  COUNT(CASE WHEN json_valid(data) AND json_type(data, '$.time.created') IN ('integer', 'real')
             THEN 1 END)
FROM session_message
WHERE type = 'assistant'";

/// SQL functions the statements above call; the authorizer rejects every other one.
pub(crate) const FUNCTIONS: [&str; 5] = ["json_extract", "json_type", "json_valid", "count", "sum"];
