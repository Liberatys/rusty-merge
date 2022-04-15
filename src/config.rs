use anyhow::Result;
use directories_next::UserDirs;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub queue: Queue,
    pub merger: Merger,
    pub notifier: Notifier,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Queue {
    pub limit: u32,
    pub interval_in_minutes: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Merger {
    pub title: Option<String>,
    pub message: Option<String>,
}

// TODO: Implement configuration for source
pub struct Source {
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Notifier {
    pub pop: Option<Notification>,
    pub merge: Option<Notification>,
    pub update: Option<Notification>,
}

// TODO: Add some methods to abstract the notification creation
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Notification {
    pub enabled: bool,
    pub title: String,
    pub message: Option<String>,
    pub icon: Option<String>,
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            limit: 10,
            interval_in_minutes: 5,
        }
    }
}

impl Default for Merger {
    fn default() -> Self {
        Self {
            title: None,
            message: None,
        }
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self {
            pop: None,
            merge: Some(Notification {
                enabled: true,
                title: String::new(),
                message: None,
                icon: None,
            }),
            update: None,
        }
    }
}

/// `Configuration` implements `Default`
impl Default for Configuration {
    fn default() -> Self {
        Self {
            queue: Queue::default(),
            merger: Merger::default(),
            notifier: Notifier::default(),
        }
    }
}

pub fn load() -> Result<Configuration> {
    let path = config_path();

    if Path::new(&path).exists() {
        let content = fs::read_to_string(path)?;
        let config: Configuration = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Configuration::default())
    }
}

pub fn store(config: &Configuration) -> Result<()> {
    let path = config_path();
    let content = toml::to_string(config).unwrap();

    ensure_folder(&path)?;

    let mut file = File::create(path)?;
    file.write(content.as_bytes())?;

    Ok(())
}

fn ensure_folder(path: &String) -> Result<()> {
    let path = std::path::Path::new(path);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix)?;

    Ok(())
}

fn config_path() -> String {
    let package_name = env!("CARGO_PKG_NAME");

    format!(
        "{}/.config/{}/config.yml",
        UserDirs::new().unwrap().home_dir().display(),
        package_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_load() -> Result<()> {
        let config = load()?;
        assert_eq!(config.merger, Merger::default());

        Ok(())
    }

    #[test]
    fn test_store() -> Result<()> {
        if Path::new(&config_path()).exists() {
            std::fs::remove_file(config_path())?;
        }

        let config = Configuration::default();

        assert_eq!(Path::new(&config_path()).exists(), false);

        store(&config)?;

        assert_eq!(Path::new(&config_path()).exists(), true);

        std::fs::remove_file(config_path())?;

        Ok(())
    }
}
