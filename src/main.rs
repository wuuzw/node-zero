use curl::easy::Easy;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use is_root::is_root;
use serde_json::Value;
use std::error::Error;
use std::io;
use std::process::exit;

mod installer;

fn get_lts_version(releases: &Value) -> String {
    let lts = releases
        .as_array()
        .unwrap()
        .iter()
        .filter(|release| release["lts"].is_string())
        .next()
        .unwrap();

    lts["version"].as_str().unwrap().to_string()
}

fn get_latest_version(releases: &Value) -> String {
    releases[0]["version"].as_str().unwrap().to_string()
}

fn get_releases() -> Value {
    let url = "https://unofficial-builds.nodejs.org/download/release/index.json";

    let mut json = Vec::new();
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                json.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    assert_eq!(200, easy.response_code().unwrap());

    let releases: Value = serde_json::from_slice(&json).unwrap();
    releases
}

fn init_config() -> Result<String, Box<dyn Error>> {
    let theme = ColorfulTheme::default();
    println!("Node.js Installer for unofficial builds");

    let releases = get_releases();
    let lts = get_lts_version(&releases);
    let latest = get_latest_version(&releases);

    let items = vec![
        format!("LTS ({})", lts),
        format!("Latest ({})", latest),
        "Specific Version".to_string(),
    ];

    let version_selection = Select::with_theme(&theme)
        .with_prompt("Which version of Nodejs do you want to install?")
        .items(&items)
        .default(0)
        .interact()?;

    let version = match version_selection {
        0 => lts,
        1 => latest,
        2 => {
            let mut specific = Input::with_theme(&theme)
                .with_prompt("Enter version number")
                .default(lts)
                .validate_with({
                    |input: &String| -> Result<(), &str> {
                        if releases
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|release| release["version"] == format!("v{}", input))
                        {
                            Ok(())
                        } else {
                            Err("Invalid version")
                        }
                    }
                })
                .interact_text()?;

            specific.insert(0, 'v');
            specific
        }
        _ => lts,
    };

    Ok(version.to_string())
}

fn main() -> io::Result<()> {
    if is_root() {
        match init_config() {
            Ok(version) => {
                let installer = installer::Installer::new();

                installer.print_sys_info();
                installer.install(version);
            }
            Err(err) => println!("error: {}", err),
        }
    } else {
        eprintln!("Run as root!");
        exit(1);
    }

    Ok(())
}
