use std::{
    env,
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use ansi_term::Colour;
use clap::{Parser, Subcommand};
use log::error;
use once_cell::sync::Lazy;

pub static DONE: Lazy<String> = Lazy::new(|| Colour::Green.paint("").to_string());
pub static SKIPPED: Lazy<String> = Lazy::new(|| Colour::Green.paint("󰒭").to_string());

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Actions,
}

#[derive(Subcommand, Debug)]
pub enum Actions {
    Init,
    Add {
        name: String,
        place: PathBuf,
    },
    Export {
        #[arg(short, long)]
        name: Option<String>,
    },
    Import {
        #[arg(short, long)]
        name: Option<String>,
    },
}

pub fn init() {
    let level = |lv: log::Level| {
        if lv == log::Level::Info {
            return Colour::Cyan.paint("+").to_string();
        }
        if lv == log::Level::Error {
            return Colour::Red.paint("x").to_string();
        }
        if lv == log::Level::Warn {
            return Colour::Yellow.paint("!").to_string();
        }
        if lv == log::Level::Debug {
            return Colour::Green.paint("%").to_string();
        }
        if lv == log::Level::Trace {
            return Colour::Purple.paint("$").to_string();
        }

        "?".to_string()
    };
    env::set_var("RUST_LOG", "debug");
    env_logger::builder()
        .format(move |buf, record| writeln!(buf, "[{}] {}", level(record.level()), record.args()))
        .init();
}

pub fn confirm(msg: &str, default: bool) -> bool {
    let mut buffer = String::new();

    let output = if default {
        "Sure? [Y/n] "
    } else {
        "Sure? [y/N] "
    };

    println!("{}", msg);
    print!("{}", output);
    if stdout().flush().is_err() {
        error!("Failed to flush stdout");
        panic!();
    }
    stdin().read_line(&mut buffer).unwrap();

    if buffer == "\n" {
        return default;
    }
    if buffer == "y\n" || buffer == "Y\n" {
        return true;
    }
    if buffer == "n\n" || buffer == "N\n" {
        return false;
    }

    error!("Invalid input");
    panic!();
}
