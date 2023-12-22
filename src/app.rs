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
            let hash = dir_hash(&place).context("failed to calculate dir hash")?;
            let new_pair = Pair { name, place, hash };
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
                    // each_pair.hashには過去に保存したときの設定ディレクトリのハッシュ値が入っている
                    // これらを比べて変更の有無を調べる。
                    if c_hash != each_pair.hash {
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
                        // ハッシュ値を更新する
                        each_pair.hash = c_hash;
                        // 設定ファイル(dotman)を更新
                        self.config
                            .update_config()
                            .context("failed to update config")?;

                        if self.config.export_hook.file_name().is_some() {
                            let mut cmd = process::Command::new("sh");
                            cmd.arg("-c");
                            cmd.arg(&self.config.export_hook);
                            cmd.spawn().with_context(|| {
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
            // 名前が指定されていないときは全ての設定を走査して、変更があったものをdotfiles配下にコピーする
            let pairs = &mut self.config.pairs;
            for each_pair in pairs {
                let c_hash = dir_hash(&each_pair.place).context("failed to calculate dir hash")?;
                if c_hash != each_pair.hash {
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

                    each_pair.hash = c_hash;
                    println!("\t{} {}", DONE.as_str(), each_pair.name);
                } else {
                    println!("\t{} {}", SKIPPED.as_str(), each_pair.name);
                    continue;
                }
            }
            // 走査が終わったら、設定ファイル(dotman)を更新
            self.config
                .update_config()
                .context("failed to update config")?;

            info!("calling hook script...");
            if self.config.export_hook.file_name().is_some() {
                let mut cmd = process::Command::new("sh");
                cmd.arg("-c");
                cmd.arg(&self.config.export_hook);
                cmd.spawn().with_context(|| {
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
        let pairs = &mut self.config.pairs;
        if let Some(name) = name {
            // 名前が指定されていたら
            // 指定された名前の設定を探す
            for each_pair in pairs {
                if name == each_pair.name {
                    let path_in_dot = each_pair.path_in_dot();
                    let c_hash = dir_hash(&path_in_dot).context("failed to calculate dir hash")?;
                    if c_hash != each_pair.hash {
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
                        each_pair.hash = c_hash;
                        self.config
                            .update_config()
                            .context("failed to update config")?;
                        if self.config.import_hook.file_name().is_some() {
                            let mut cmd = process::Command::new(&self.config.import_hook);
                            cmd.spawn().with_context(|| {
                                format!(
                                    "\"import\" hook script {} failed to start",
                                    self.config.import_hook.display()
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
            for each_pair in pairs {
                let path_in_dot = each_pair.path_in_dot();
                let c_hash = dir_hash(&path_in_dot).context("failed to calculate dir hash")?;
                if c_hash != each_pair.hash {
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
                    each_pair.hash = c_hash;
                    println!("\t{} {}", DONE.as_str(), each_pair.name);
                } else {
                    println!("\t{} {}", SKIPPED.as_str(), each_pair.name);
                    continue;
                }
            }
            self.config
                .update_config()
                .context("failed to update config")?;
            info!("calling hook script...");
            if self.config.import_hook.file_name().is_some() {
                let mut cmd = process::Command::new(&self.config.import_hook);
                cmd.spawn().with_context(|| {
                    format!(
                        "\"import\" hook script {} failed to start",
                        self.config.import_hook.display()
                    )
                })?;
            }
        }
        Ok(())
    }
}
