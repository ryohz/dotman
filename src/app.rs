use std::{fs, io, path::PathBuf, process};

use anyhow::{anyhow, bail, Context, Result};
use log::info;

use crate::{
    cli::{DONE, SKIPPED},
    config::{self, Pair},
    util::{copy_directory, dir_hash, is_exists},
};

pub struct App {
    config: config::Config,
}

impl App {
    #[allow(dead_code)]
    pub fn new(config: config::Config) -> Self {
        Self { config }
    }

    /// 新たな設定を設定を追加する関数。
    /// すでにある名前と場所を追加することは許されない。
    pub fn add_config(&mut self, name: String, place: PathBuf) -> Result<()> {
        if is_exists(&place).context("failed to check if place exists")? {
            if self.config.conflict_check(name.clone(), place.clone()) {
                bail!(anyhow!(
                    "name {} or path {} is already registered",
                    name,
                    place.display()
                ));
            }
            let new_pair = Pair { name, place };
            let path_in_dot = &new_pair.path_in_dot();
            match fs::remove_dir_all(path_in_dot) {
                Ok(_) => {}
                Err(e) => match e.kind() {
                    io::ErrorKind::NotFound => {}
                    _ => bail!(anyhow!(
                        "failed to remove directory {}",
                        path_in_dot.display()
                    )),
                },
            }
            copy_directory(&new_pair.place, path_in_dot).with_context(|| {
                format!("failed to copy directory {}", new_pair.place.display())
            })?;
            self.config.pairs.push(new_pair);
            self.config
                .update_config()
                .context("failed to update config")?;

            Ok(())
        } else {
            Err(anyhow!("place {} does not exist", place.display()))
        }
    }

    /// 与えられた名前の設定が変更されていた場合、その設定をdotfiles配下にコピー(上書き)する
    /// 名前が与えられなかった場合、全ての設定を走査して、登録された設定が変更されていた場合は、その設定をdotfiles配下にコピーする
    pub fn export_config(&mut self, name: Option<String>) -> Result<()> {
        if let Some(name) = name {
            // 名前が指定されているとき
            let pairs = &mut self.config.pairs;
            for each_pair in pairs {
                // 登録されている全ての設定を走査して指定された名前の設定を探す
                if name == each_pair.name {
                    // 名前が見つかったら、過去から変更があるか調べて、あったらそれをdotfiles配下にコピーする
                    // なければ関数を終了する
                    // 現在の設定ディレクトリのハッシュ値を計算
                    let c_hash =
                        dir_hash(&each_pair.place).context("failed to calculate dir hash")?;
                    let p_hash = dir_hash(&each_pair.path_in_dot())
                        .context("failed to calculate dir hash")?;
                    if c_hash != p_hash {
                        // 変更が検知されたら
                        // 設定ディレクトリをdotfiles配下にコピーする処理をする
                        let path_in_dot = each_pair.path_in_dot();
                        // まずもともとあるdotfiles配下にある過去の設定ディレクトリを削除
                        match fs::remove_dir_all(&path_in_dot) {
                            Ok(_) => {}
                            Err(e) => match e.kind() {
                                io::ErrorKind::NotFound => {}
                                _ => bail!(anyhow!(
                                    "failed to remove directory {}",
                                    path_in_dot.display()
                                )),
                            },
                        }
                        // 次に、設定ディレクトリをdotfiles配下にコピー
                        copy_directory(&each_pair.place, &path_in_dot).with_context(|| {
                            format!(
                                "failed to copy directory {} to {}",
                                each_pair.place.display(),
                                path_in_dot.display()
                            )
                        })?;

                        if self.config.export_hook.file_name().is_some() {
                            let mut cmd = process::Command::new("sh");
                            cmd.arg("-c");
                            cmd.arg(&self.config.export_hook);
                            cmd.output().with_context(|| {
                                format!(
                                    "\"export\" hook script {} failed to start",
                                    self.config.export_hook.display()
                                )
                            })?;
                        }
                        println!("\t{} {}", DONE.as_str(), name);
                        return Ok(());
                    } else {
                        // 変更が検知されなかった場合
                        println!("\t{} {}", SKIPPED.as_str(), name);
                        return Ok(());
                    }
                }
            }
            // 指定された名前の設定が見つからなかった場合
            bail!(anyhow!("name {} not found", name));
        } else {
            // export hookを実行するかを判断するためのフラグ
            // １つでも設定が更新された場合はフラグが立つ
            let mut change_flag = false;
            // 名前が指定されていないときは全ての設定を走査して、変更があったものをdotfiles配下にコピーする
            let pairs = &mut self.config.pairs;
            for each_pair in pairs {
                let c_hash = dir_hash(&each_pair.place).context("failed to calculate dir hash")?;
                let p_hash =
                    dir_hash(&each_pair.path_in_dot()).context("failed to calculate dir hash")?;
                if c_hash != p_hash {
                    let path_in_dot = each_pair.path_in_dot();
                    match fs::remove_dir_all(&path_in_dot) {
                        Ok(_) => {}
                        Err(e) => match e.kind() {
                            io::ErrorKind::NotFound => {}
                            _ => bail!(anyhow!(
                                "failed to remove directory {}",
                                path_in_dot.display()
                            )),
                        },
                    }
                    copy_directory(&each_pair.place, &path_in_dot).with_context(|| {
                        format!(
                            "failed to copy directory {} to {}",
                            each_pair.place.display(),
                            path_in_dot.display()
                        )
                    })?;

                    change_flag = true;
                    println!("\t{} {}", DONE.as_str(), each_pair.name);
                } else {
                    println!("\t{} {}", SKIPPED.as_str(), each_pair.name);
                    continue;
                }
            }

            if change_flag && self.config.export_hook.file_name().is_some() {
                info!("calling hook script...");
                let mut cmd = process::Command::new("sh");
                cmd.arg("-c");
                cmd.arg(&self.config.export_hook);
                cmd.output().with_context(|| {
                    format!(
                        "\"export\" hook script {} failed to start",
                        self.config.export_hook.display()
                    )
                })?;
            }
            Ok(())
        }
    }

    pub fn import_config(&mut self, name: Option<String>) -> Result<()> {
        if self.config.before_import_hook.file_name().is_some() {
            info!("calling hook script before import...");
            let mut cmd = process::Command::new("sh");
            cmd.arg("-c");
            cmd.arg(&self.config.before_import_hook);
            cmd.output().with_context(|| {
                format!(
                    "\"import\" hook script {} failed to start",
                    self.config.before_import_hook.display()
                )
            })?;
        }
        let pairs = &mut self.config.pairs;
        if let Some(name) = name {
            // 名前が指定されていたら
            // 指定された名前の設定を探す
            for each_pair in pairs {
                if name == each_pair.name {
                    let path_in_dot = each_pair.path_in_dot();
                    if !is_exists(&each_pair.place)
                        .context("failed to check if directory exists")?
                    {
                        fs::create_dir_all(&each_pair.place)
                            .context("failed to create directory")?;
                    }
                    let c_hash =
                        dir_hash(&each_pair.place).context("failed to calculate dir hash")?;
                    let p_hash = dir_hash(&path_in_dot).context("failed to calculate dir hash")?;
                    if c_hash != p_hash {
                        match fs::remove_dir_all(&each_pair.place) {
                            Ok(_) => {}
                            Err(e) => match e.kind() {
                                io::ErrorKind::NotFound => fs::create_dir_all(&each_pair.place)
                                    .with_context(|| {
                                        format!(
                                            "failed to create directory {}",
                                            each_pair.place.display()
                                        )
                                    })?,
                                _ => {
                                    bail!(anyhow!(
                                        "failed to remove directory {}",
                                        each_pair.place.display()
                                    ));
                                }
                            },
                        }
                        copy_directory(&path_in_dot, &each_pair.place).with_context(|| {
                            format!(
                                "failed to copy directory {} to {}",
                                path_in_dot.display(),
                                each_pair.place.display()
                            )
                        })?;
                        if self.config.after_import_hook.file_name().is_some() {
                            info!("calling hook script after import...");
                            let mut cmd = process::Command::new("sh");
                            cmd.arg("-c");
                            cmd.arg(&self.config.after_import_hook);
                            cmd.output().with_context(|| {
                                format!(
                                    "\"import\" hook script {} failed to start",
                                    self.config.after_import_hook.display()
                                )
                            })?;
                        }

                        println!("\t{} {}", DONE.as_str(), name);
                        return Ok(());
                    } else {
                        println!("\t{} {}", SKIPPED.as_str(), name);
                        return Ok(());
                    }
                }
            }
        } else {
            let mut change_flag = false;
            for each_pair in pairs {
                let path_in_dot = each_pair.path_in_dot();
                if !is_exists(&each_pair.place).context("failed to check if directory exists")? {
                    fs::create_dir_all(&each_pair.place).context("failed to create directory")?;
                }
                let c_hash = dir_hash(&each_pair.place).context("failed to calculate dir hash")?;
                let p_hash = dir_hash(&path_in_dot).context("failed to calculate dir hash")?;
                if c_hash != p_hash {
                    match fs::remove_dir_all(&each_pair.place) {
                        Ok(_) => {}
                        Err(e) => match e.kind() {
                            io::ErrorKind::NotFound => fs::create_dir_all(&each_pair.place)
                                .with_context(|| {
                                    format!(
                                        "failed to create directory {}",
                                        each_pair.place.display()
                                    )
                                })?,
                            _ => {
                                bail!(anyhow!(
                                    "failed to remove directory {}",
                                    each_pair.place.display()
                                ));
                            }
                        },
                    }
                    copy_directory(&path_in_dot, &each_pair.place).with_context(|| {
                        format!(
                            "failed to copy directory {} to {}",
                            path_in_dot.display(),
                            each_pair.place.display()
                        )
                    })?;
                    change_flag = true;
                    println!("\t{} {}", DONE.as_str(), each_pair.name);
                } else {
                    println!("\t{} {}", SKIPPED.as_str(), each_pair.name);
                    continue;
                }
            }
            if change_flag && self.config.after_import_hook.file_name().is_some() {
                info!("calling hook script after import...");
                let mut cmd = process::Command::new("sh");
                cmd.arg("-c");
                cmd.arg(&self.config.after_import_hook);
                cmd.output().with_context(|| {
                    format!(
                        "\"import\" hook script {} failed to start",
                        self.config.after_import_hook.display()
                    )
                })?;
            }
        }
        Ok(())
    }
}
