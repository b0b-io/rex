use super::*;
use crate::config;
use crate::format::{self, OutputFormat};

/// Handle the registry init subcommand
pub fn handle_registry_init(ctx: &crate::context::AppContext, name: &str, url: &str) {
    let config_path = config::get_config_path();
    match init_registry(&config_path, name, url) {
        Ok(_) => format::success(ctx, &format!("Initialized registry '{}' at {}", name, url)),
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry remove subcommand
pub fn handle_registry_remove(ctx: &crate::context::AppContext, name: &str, force: bool) {
    let config_path = config::get_config_path();
    match remove_registry(&config_path, name, force) {
        Ok(_) => format::success(ctx, &format!("Removed registry '{}'", name)),
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry use subcommand
pub fn handle_registry_use(ctx: &crate::context::AppContext, name: &str) {
    let config_path = config::get_config_path();
    match use_registry(&config_path, name) {
        Ok(_) => format::success(ctx, &format!("Set '{}' as default registry", name)),
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry show subcommand
pub fn handle_registry_show(ctx: &crate::context::AppContext, name: &str, format: OutputFormat) {
    let config_path = config::get_config_path();
    match show_registry(&config_path, name) {
        Ok(registry) => match crate::format::format_output(&registry, format) {
            Ok(output) => println!("{}", output),
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

/// Handle the registry list subcommand
pub fn handle_registry_list(ctx: &crate::context::AppContext, format: OutputFormat) {
    let config_path = config::get_config_path();
    match list_registries(&config_path) {
        Ok(registries) => {
            if registries.is_empty() {
                println!("No registries configured.");
                return;
            }

            match format {
                OutputFormat::Pretty => {
                    use tabled::Table;
                    let table = Table::new(&registries).to_string();
                    println!("{}", table);
                }
                OutputFormat::Json => match serde_json::to_string_pretty(&registries) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        format::error(ctx, &format!("formatting JSON: {}", e));
                        std::process::exit(1);
                    }
                },
                OutputFormat::Yaml => match serde_yaml::to_string(&registries) {
                    Ok(yaml) => print!("{}", yaml),
                    Err(e) => {
                        format::error(ctx, &format!("formatting YAML: {}", e));
                        std::process::exit(1);
                    }
                },
            }
        }
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry check subcommand
pub async fn handle_registry_check(
    ctx: &crate::context::AppContext,
    name: &str,
    format: OutputFormat,
) {
    let config_path = config::get_config_path();
    let result = check_registry(&config_path, name).await;

    match crate::format::format_output(&result, format) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            format::error(ctx, &format!("formatting output: {}", e));
            std::process::exit(1);
        }
    }
}

/// Handle the registry login subcommand
pub async fn handle_registry_login(
    ctx: &crate::context::AppContext,
    name: &str,
    username: Option<&str>,
    password: Option<&str>,
) {
    let config_path = config::get_config_path();

    match login_registry(&config_path, name, username, password).await {
        Ok(_) => format::success(ctx, &format!("Stored credentials for '{}'", name)),
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry logout subcommand
pub fn handle_registry_logout(ctx: &crate::context::AppContext, name: &str) {
    let config_path = config::get_config_path();

    match logout_registry(&config_path, name) {
        Ok(_) => format::success(ctx, &format!("Logged out from '{}'", name)),
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the cache stats subcommand
pub fn handle_cache_stats(
    ctx: &crate::context::AppContext,
    name: Option<&str>,
    format: OutputFormat,
) {
    let config_path = config::get_config_path();

    match cache_stats(&config_path, name) {
        Ok(stats) => match crate::format::format_output(&stats, format) {
            Ok(output) => println!("{}", output),
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

/// Handle the cache clear subcommand
pub fn handle_cache_clear(
    ctx: &crate::context::AppContext,
    name: Option<&str>,
    all: bool,
    force: bool,
) {
    let config_path = config::get_config_path();

    match cache_clear(&config_path, name, all, force) {
        Ok(stats) => {
            println!(
                "{} Cleared {} entries ({} bytes)",
                format::checkmark(ctx),
                stats.removed_files,
                stats.reclaimed_space
            );
            format::success(ctx, "Cache cleared successfully");
        }
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the cache prune subcommand
pub fn handle_cache_prune(
    ctx: &crate::context::AppContext,
    name: Option<&str>,
    all: bool,
    dry_run: bool,
) {
    let config_path = config::get_config_path();

    match cache_prune(&config_path, name, all, dry_run) {
        Ok(stats) => {
            if dry_run {
                println!(
                    "Would remove {} expired entries ({} bytes)",
                    stats.removed_files, stats.reclaimed_space
                );
            } else {
                format::success(
                    ctx,
                    &format!("Removed {} expired entries", stats.removed_files),
                );
                format::success(
                    ctx,
                    &format!("Freed {} bytes of disk space", stats.reclaimed_space),
                );
            }
        }
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

/// Handle the cache sync subcommand
pub async fn handle_cache_sync(
    ctx: &crate::context::AppContext,
    name: Option<&str>,
    manifests: bool,
    all: bool,
    force: bool,
) {
    let config_path = config::get_config_path();

    match cache_sync(ctx, &config_path, name, manifests, all, force).await {
        Ok(stats) => {
            format::success(ctx, "Cache synced successfully:");
            println!("  {} catalog entries", stats.catalog_entries);
            println!("  {} tag entries", stats.tag_entries);
            if manifests {
                println!("  {} manifest entries", stats.manifest_entries);
            }
            println!(
                "  Total size: {:.2} MB",
                stats.total_size as f64 / 1_048_576.0
            );
        }
        Err(e) => {
            format::error(ctx, &e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod tests;
