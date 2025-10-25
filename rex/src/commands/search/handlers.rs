use super::*;
use crate::format::OutputFormat;

/// Handle the search command
pub async fn handle_search(query: &str, format: OutputFormat, limit: Option<usize>) {
    match search(query, limit).await {
        Ok(results) => match crate::format::format_output(&results, format) {
            Ok(output) => print!("{}", output),
            Err(e) => {
                eprintln!("Error formatting output: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
