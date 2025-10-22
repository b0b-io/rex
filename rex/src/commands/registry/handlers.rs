use super::*;
use crate::config;
use crate::format::OutputFormat;

/// Handle the registry init subcommand
pub fn handle_registry_init(name: &str, url: &str) {
    let config_path = config::get_config_path();
    match init_registry(&config_path, name, url) {
        Ok(_) => println!("Initialized registry '{}' at {}", name, url),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry remove subcommand
pub fn handle_registry_remove(name: &str) {
    let config_path = config::get_config_path();
    match remove_registry(&config_path, name) {
        Ok(_) => println!("Removed registry '{}'", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry use subcommand
pub fn handle_registry_use(name: &str) {
    let config_path = config::get_config_path();
    match use_registry(&config_path, name) {
        Ok(_) => println!("Set '{}' as default registry", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry show subcommand
pub fn handle_registry_show(name: &str, format: OutputFormat) {
    let config_path = config::get_config_path();
    match show_registry(&config_path, name) {
        Ok(registry) => match crate::format::format_output(&registry, format) {
            Ok(output) => println!("{}", output),
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

/// Handle the registry list subcommand
pub fn handle_registry_list(format: OutputFormat) {
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
                        eprintln!("Error formatting JSON: {}", e);
                        std::process::exit(1);
                    }
                },
                OutputFormat::Yaml => match serde_yaml::to_string(&registries) {
                    Ok(yaml) => print!("{}", yaml),
                    Err(e) => {
                        eprintln!("Error formatting YAML: {}", e);
                        std::process::exit(1);
                    }
                },
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry check subcommand
pub async fn handle_registry_check(name: &str, format: OutputFormat) {
    let config_path = config::get_config_path();
    let result = check_registry(&config_path, name).await;

    match crate::format::format_output(&result, format) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("Error formatting output: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry login subcommand
pub async fn handle_registry_login(name: &str, username: Option<&str>, password: Option<&str>) {
    let config_path = config::get_config_path();

    match login_registry(&config_path, name, username, password).await {
        Ok(_) => println!("Successfully stored credentials for '{}'", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle the registry logout subcommand
pub fn handle_registry_logout(name: &str) {
    let config_path = config::get_config_path();

    match logout_registry(&config_path, name) {
        Ok(_) => println!("Successfully logged out from '{}'", name),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod tests;
