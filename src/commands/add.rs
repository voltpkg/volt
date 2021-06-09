/*
Copyright 2021 Volt Contributors
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! Add a package to your dependencies for your project.

use std::collections::HashMap;
// Std Imports
use std::{process::exit, sync::atomic::AtomicI16};
// use std::sync::atomic::Ordering;
use std::sync::Arc;

// Library Imports
use anyhow::{Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use futures::{stream::FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{
    self,
    sync::{mpsc, Mutex},
};

use std::io::Write;

use crate::commands::init;

use crate::classes::package::PackageJson;
use crate::model::lock_file::{DependencyID, DependencyLock};
// Crate Level Imports
use crate::utils;
use crate::utils::download_tarball;
use crate::utils::App;
use crate::VERSION;
use crate::{
    classes::package::{Package, Version},
    utils::PROGRESS_CHARS,
};
use crate::{classes::voltapi::VoltPackage, model::lock_file::LockFile};

// Super Imports
use super::Command;

/// Struct implementation for the `Add` command.
#[derive(Clone)]
pub struct Add {
    lock_file: LockFile,
    dependencies: Arc<Mutex<Vec<(Package, Version)>>>,
    total_dependencies: Arc<AtomicI16>,
    progress_sender: mpsc::Sender<()>,
}

#[async_trait]
impl Command for Add {
    /// Display a help menu for the `volt add` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Add a package to your dependencies for your project.
Usage: {} {} {} {}
Options: 
    
  {} {} Output the version number.
  {} {} Output verbose messages on internal operations.
  {} {} Disable progress bar."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "add".bright_purple(),
            "[packages]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    /// Execute the `volt add` command
    ///
    /// Adds a package to dependencies for your project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Add react to your dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app, vec!["react"], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        if app.args.len() == 1 {
            println!("{}", Self::help());
            exit(1);
        }

        let mut packages = vec![];
        for arg in &app.args {
            if arg != "add" {
                packages.push(arg.clone());
            }
        }

        if !std::env::current_dir()?.join("package.json").exists() {
            println!("{} no package.json found", "error".bright_red());
            print!("Do you want to initialize package.json (Y/N): ");
            std::io::stdout().flush().expect("Could not flush stdout");
            let mut string: String = String::new();
            let _ = std::io::stdin().read_line(&mut string);
            if string.trim().to_lowercase() != "y" {
                exit(0);
            } else {
                init::Init::exec(app.clone()).await.unwrap();
            }
        }

        let package_file = Arc::new(Mutex::new(PackageJson::from("package.json")));
        let mut handles = vec![];

        println!("{}", "Adding dependencies".bright_purple());

        for package in packages.clone() {
            let app_new = app.clone();

            let package_dir_loc;

            if cfg!(windows) {
                package_dir_loc = format!(
                    r"{}\.volt\{}",
                    std::env::var("USERPROFILE").unwrap(),
                    package
                );
            } else {
                package_dir_loc = format!(r"{}\.volt\{}", std::env::var("HOME").unwrap(), package);
            }

            let package_dir = std::path::Path::new(&package_dir_loc);
            let package_file = package_file.clone();

            if package_dir.exists() {
                handles.push(tokio::spawn(async move {
                    let verbose = app_new.has_flag(&["-v", "--verbose"]);
                    let pballowed = !app_new.has_flag(&["--no-progress", "-np"]);

                    let mut lock_file = LockFile::load(app_new.lock_file_path.to_path_buf())
                        .unwrap_or_else(|_| LockFile::new(app_new.lock_file_path.to_path_buf()));

                    // TODO: Change this to handle multiple packages
                    let progress_bar: ProgressBar = ProgressBar::new(1);

                    progress_bar.set_style(
                        ProgressStyle::default_bar()
                            .progress_chars(PROGRESS_CHARS)
                            .template(&format!(
                                "{} [{{bar:40.magenta/blue}}] {{msg:.blue}}",
                                "Fetching dependencies".bright_blue()
                            )),
                    );

                    let response = utils::get_volt_response(package.to_string()).await;

                    let progress_bar = &progress_bar;

                    progress_bar.finish_with_message("[OK]".bright_green().to_string());

                    let length = &response
                        .versions
                        .get(&response.version)
                        .unwrap()
                        .packages
                        .len();

                    if *length == 1 {
                        println!("Loaded 1 dependency");
                    } else {
                        println!("Loaded {} dependencies.", length);
                    }

                    let current_version = response.versions.get(&response.version).unwrap();

                    let dependencies: Vec<_> = current_version
                        .packages
                        .iter()
                        .map(|(_, object)| {
                            let mut lock_dependencies: HashMap<String, String> = HashMap::new();

                            if object.clone().dependencies.is_some() {
                                for dep in object.clone().dependencies.unwrap().iter() {
                                    // TODO: Change this to real version
                                    lock_dependencies.insert(dep.clone(), String::new());
                                }
                            }

                            lock_file.dependencies.insert(
                                DependencyID(object.clone().name, object.clone().version),
                                DependencyLock {
                                    name: object.clone().name,
                                    version: object.clone().version,
                                    tarball: object.clone().tarball,
                                    sha1: object.clone().sha1,
                                    dependencies: lock_dependencies,
                                },
                            );

                            object.clone()
                        })
                        .collect();

                    let mut workers = FuturesUnordered::new();

                    for dep in dependencies.clone() {
                        let app_new = app_new.clone();
                        workers.push(async move {
                            Add::install_extract_package(app_new, &dep).await.unwrap();
                            utils::generate_script(&dep);
                        });
                    }

                    if pballowed {
                        let progress_bar = ProgressBar::new(workers.len() as u64);

                        progress_bar.set_style(
                            ProgressStyle::default_bar()
                                .progress_chars(PROGRESS_CHARS)
                                .template(&format!(
                                    "{} [{{bar:40.magenta/blue}}] {{msg:.blue}} {{pos}} / {{len}}",
                                    "Installing packages".bright_blue()
                                )),
                        );

                        while workers.next().await.is_some() {
                            progress_bar.inc(1);
                        }

                        progress_bar.finish();
                    } else {
                        while workers.next().await.is_some() {}
                    }

                    for dep in dependencies {
                        if dep.name == package {
                            utils::create_dep_symlinks(
                                package.as_str(),
                                current_version.packages.clone(),
                            )
                            .await
                            .unwrap();
                        }
                    }

                    // Change package.json
                    // for value in &dependencies.to_owned().iter() {
                    //     package_file.add_dependency(value.0.name, value.1.version);
                    // }

                    let mut package_json_file = package_file.lock().await;

                    package_json_file
                        .dependencies
                        .insert(package.to_string(), response.clone().version);

                    package_json_file.save();

                    // Write to lock file
                    if verbose {
                        println!("info {}", "Writing to lock file".yellow());
                    }

                    lock_file
                        .save()
                        .context("Failed to save lock file")
                        .unwrap();
                }));
            } else {
                let verbose = app_new.has_flag(&["-v", "--verbose"]);
                let pballowed = !app_new.has_flag(&["--no-progress", "-np"]);

                let mut lock_file = LockFile::load(app_new.lock_file_path.to_path_buf())
                    .unwrap_or_else(|_| LockFile::new(app_new.lock_file_path.to_path_buf()));

                // TODO: Change this to handle multiple packages
                let progress_bar: ProgressBar = ProgressBar::new(1);

                progress_bar.set_style(
                    ProgressStyle::default_bar()
                        .progress_chars(PROGRESS_CHARS)
                        .template(&format!(
                            "{} [{{bar:40.magenta/blue}}] {{msg:.blue}}",
                            "Fetching dependencies".bright_blue()
                        )),
                );

                let response = utils::get_volt_response(package.to_string()).await;

                let progress_bar = &progress_bar;

                progress_bar.finish_with_message("[OK]".bright_green().to_string());

                let length = &response
                    .versions
                    .get(&response.version)
                    .unwrap()
                    .packages
                    .len();

                if *length == 1 {
                    println!("Loaded 1 dependency");
                } else {
                    println!("Loaded {} dependencies.", length);
                }

                let current_version = response.versions.get(&response.version).unwrap();

                let dependencies: Vec<_> = current_version
                    .packages
                    .iter()
                    .map(|(_, object)| {
                        let mut lock_dependencies: HashMap<String, String> = HashMap::new();

                        if object.clone().dependencies.is_some() {
                            for dep in object.clone().dependencies.unwrap().iter() {
                                // TODO: Change this to real version
                                lock_dependencies.insert(dep.clone(), String::new());
                            }
                        }

                        lock_file.dependencies.insert(
                            DependencyID(object.clone().name, object.clone().version),
                            DependencyLock {
                                name: object.clone().name,
                                version: object.clone().version,
                                tarball: object.clone().tarball,
                                sha1: object.clone().sha1,
                                dependencies: lock_dependencies,
                            },
                        );

                        object.clone()
                    })
                    .collect();

                let mut workers = FuturesUnordered::new();

                for dep in dependencies.clone() {
                    let app_new = app_new.clone();
                    workers.push(async move {
                        Add::install_extract_package(app_new, &dep).await.unwrap();
                        utils::generate_script(&dep);
                    });
                }

                if pballowed {
                    let progress_bar = ProgressBar::new(workers.len() as u64);

                    progress_bar.set_style(
                        ProgressStyle::default_bar()
                            .progress_chars(PROGRESS_CHARS)
                            .template(&format!(
                                "{} [{{bar:40.magenta/blue}}] {{msg:.blue}} {{pos}} / {{len}}",
                                "Installing packages".bright_blue()
                            )),
                    );

                    while workers.next().await.is_some() {
                        progress_bar.inc(1);
                    }

                    progress_bar.finish();
                } else {
                    while workers.next().await.is_some() {
                        progress_bar.inc(1);
                    }
                }

                for dep in dependencies {
                    if dep.name == package {
                        utils::create_dep_symlinks(
                            package.as_str(),
                            current_version.packages.clone(),
                        )
                        .await
                        .unwrap();
                    }
                }

                // Change package.json
                // for value in &dependencies.to_owned().iter() {
                //     package_file.add_dependency(value.0.name, value.1.version);
                // }

                // Write to lock file
                if verbose {
                    println!("info {}", "Writing to lock file".yellow());
                }

                lock_file
                    .save()
                    .context("Failed to save lock file")
                    .unwrap();
            }
        }

        if !handles.is_empty() {
            for handle in handles {
                handle.await?;
            }
        }

        Ok(())
    }
}

impl Add {
    // Add new package
    async fn install_extract_package(app: Arc<App>, package: &VoltPackage) -> Result<()> {
        let pb = ProgressBar::new(0);
        let text = format!("{}", "Installing Packages".bright_cyan());

        pb.set_style(
            ProgressStyle::default_spinner()
                .template(("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str())
                .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
        );

        let tarball_path = download_tarball(&app, &package).await?;

        app.extract_tarball(&tarball_path, &package)
            .await
            .with_context(|| {
                format!("Unable to extract tarball for package '{}'", &package.name)
            })?;

        utils::generate_script(package);

        println!("{}", "Successfully Added Packages".bright_blue());

        Ok(())
    }
}
