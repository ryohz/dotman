mod app;
mod cli;
mod config;
mod util;

use crate::app::App;

use clap::Parser;
use log::{error, info};

use crate::config::Config;

fn main() {
    cli::init();
    let args = cli::Args::parse();

    match args.action {
        cli::Actions::Init => {
            info!("initializing config...");
            let confirm = cli::confirm("it will overwrite existing config file", false);
            if confirm {
                match Config::init() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("failed to init config: {:#}\n", e);
                    }
                }
                info!("done");
            } else {
                info!("aborted");
            }
        }
        cli::Actions::Add { name, place } => {
            info!("adding config...");
            let config = match Config::read_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("failed to read config: {:#}\n", e);
                    return;
                }
            };
            let mut app = App::new(config);
            if let Err(e) = app.add_config(name, place) {
                error!("failed to add config: {:#}\n", e);
            }
            info!("done");
        }
        cli::Actions::Export { name } => {
            info!("exporting config...");
            let config = match Config::read_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("failed to read config: {:#}\n", e);
                    return;
                }
            };
            let mut app = App::new(config);
            if let Err(e) = app.export_config(name) {
                error!("failed to export config: {:#}\n", e);
            }
            info!("done");
        }
        cli::Actions::Import { name } => {
            info!("importing config...");
            let config = match Config::read_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("failed to read config: {:#}\n", e);
                    return;
                }
            };
            let mut app = App::new(config);
            if let Err(e) = app.import_config(name) {
                error!("failed to import config: {:#}\n", e);
            }
            info!("done");
        }
    }
}
