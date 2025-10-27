use super::*;
use crate::format::{self, OutputFormat};

/// Handle the search command
pub async fn handle_search(
    ctx: &crate::context::AppContext,
    query: &str,
    format: OutputFormat,
    limit: Option<usize>,
) {
    match search(ctx, query, limit).await {
        Ok(results) => match crate::format::format_output(&results, format) {
            Ok(output) => print!("{}", output),
            Err(e) => {
                format::error(ctx, &format!("formatting output: {}", e));
                std::process::exit(1);
            }
        },
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}
