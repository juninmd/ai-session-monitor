//! Deny-by-default SQLite authorizer: the file also holds credentials, accounts and prompts, so
//! the connection can only read the two `session_message` columns the aggregate needs. Even a
//! mistake in the SQL text, or a view that points elsewhere, cannot reach another table.

use rusqlite::hooks::{AuthAction, AuthContext, Authorization};

use super::sql;

/// The only columns the aggregate touches: the message kind and the JSON it extracts numbers from.
const READABLE_COLUMNS: [&str; 2] = ["type", "data"];

pub(crate) fn authorize(ctx: AuthContext<'_>) -> Authorization {
    if ctx.database_name.is_some_and(|name| !name.eq_ignore_ascii_case("main")) {
        return Authorization::Deny;
    }
    let allowed = match ctx.action {
        AuthAction::Select => true,
        AuthAction::Read { table_name, column_name } => {
            table_name == sql::TABLE && READABLE_COLUMNS.contains(&column_name)
        }
        AuthAction::Function { function_name } => {
            sql::FUNCTIONS.iter().any(|name| name.eq_ignore_ascii_case(function_name))
        }
        AuthAction::Pragma { pragma_name, pragma_value } => {
            pragma_name.eq_ignore_ascii_case("table_info") && pragma_value == Some(sql::TABLE)
        }
        _ => false,
    };
    if allowed { Authorization::Allow } else { Authorization::Deny }
}
