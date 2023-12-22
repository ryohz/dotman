use std::fs;
use std::io;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use dirs::home_dir;
use serde::Deserialize;
use serde::Serialize;

pub static CONFIG_DIR: once_cell::sync::Lazy<PathBuf> = once_cell::sync::Lazy::new(|| {
    if let Some(home) = home_dir() {
        home.join("dotfiles")
    } else {
        panic!("failed to get home directory");
    }
});
static CONFIG_FILE: &str = "dotman.toml";

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub pairs: Vec<Pair>,
    pub import_hook: PathBuf,
    pub export_hook: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Pair {
    pub name: String,
    pub place: PathBuf,
    pub hash: u32,
}

impl Config {
    pub fn init() -> Result<Self> {
        let config_dir = CONFIG_DIR.to_path_buf();
        let config = Self {
            pairs: vec![],
            import_hook: PathBuf::new(),
            export_hook: PathBuf::new(),
        };
        let config_path = config_dir.join(CONFIG_FILE);

        // dotfiles
        if let Err(e) = fs::remove_dir_all(&config_dir) {
            if e.kind() != io::ErrorKind::NotFound {
                bail!(anyhow!("failed to remove config directory: {}", e));
            }
        }
        fs::create_dir_all(&config_dir).with_context(|| {
            format!("failed to create config directory {}", config_dir.display())
        })?;

        // dotfiles/dotman.toml
        fs::File::create(&config_path)
            .with_context(|| format!("failed to create config file {}", config_path.display()))?;

        fs::write(
            &config_path,
            toml::to_string(&config).context("failed to serialize config as toml file")?,
        )
        .context("failed to write config to file")?;

        Ok(config)
    }

    pub fn read_config() -> Result<Self> {
        let config_dir = CONFIG_DIR.to_path_buf();
        let config_path = config_dir.join(CONFIG_FILE);

        let file = fs::File::open(&config_path)
            .with_context(|| format!("failed to open config file {}", config_path.display()))?;
        let mut reader = io::BufReader::new(file);

        let mut toml_content = String::new();
        reader
            .read_to_string(&mut toml_content)
            .with_context(|| format!("failed to read config file {}", config_path.display()))?;

        let config =
            toml::from_str::<Config>(&toml_content).context("failed to parse toml as config")?;

        Ok(config)
    }

    pub fn conflict_check(&self, name: String, place: PathBuf) -> bool {
        for each_pair in &self.pairs {
            if name == each_pair.name || place == each_pair.place {
                return true;
            }
        }
        false
    }

    pub fn update_config(&self) -> Result<()> {
        let config_dir = CONFIG_DIR.to_path_buf();
        let config_path = config_dir.join(CONFIG_FILE);

        let file = fs::File::create(&config_path)
            .with_context(|| format!("failed to open config file {}", config_path.display()))?;
        let mut writer = BufWriter::new(file);
        let toml_content = toml::to_string(&self).context("failed to serialize config as toml")?;
        writer
            .write(toml_content.as_bytes())
            .context("failed to write config to file")?;
        Ok(())
    }
}

impl Pair {
    pub fn path_in_dot(&self) -> PathBuf {
        CONFIG_DIR.join(&self.name)
    }
}
