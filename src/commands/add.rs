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

//! Add a package to the dependencies for your project.

use std::{collections::HashMap, time::Instant};

use crate::{
    cli::{VoltCommand, VoltConfig},
    core::net::fetch_dep_tree,
    core::utils::voltapi::VoltPackage,
    core::utils::{install_package, State},
};

use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use futures::{stream::FuturesUnordered, StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use miette::IntoDiagnostic;
use package_spec::PackageSpec;
use reqwest::Client;

/// Add a package to your project's dependencies
#[derive(Debug, Parser)]
pub struct Add {
    /// Packages to add to the dependencies for your project.
    packages: Vec<PackageSpec>,
}

#[async_trait]
impl VoltCommand for Add {
    async fn exec(self, config: VoltConfig) -> miette::Result<()> {
        // let global_lock_file = LockFile::load(config.home()?.join(".global.lock"), true).unwrap();

        // let local_lock_file =

        let bar = ProgressBar::new_spinner()
            .with_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}"));

        bar.enable_steady_tick(10);

        let resolve_start = Instant::now();

        let mut requested_packages = vec![];

        // Fetch pre-flattened dependency trees from the registry
        let responses = fetch_dep_tree(&self.packages, &bar).await?;

        let mut tree: HashMap<String, VoltPackage> = HashMap::new();

        for response in responses {
            let _index = 0;

            for package in &self.packages {
                if let PackageSpec::Npm {
                    name,
                    scope,
                    requested,
                } = package
                {
                    // receive the version of a package that has been requested from the response
                    if *name == response.name {
                        requested_packages.push(PackageSpec::Npm {
                            scope: scope.to_owned(),
                            name: name.to_owned(),
                            requested: Some(package_spec::VersionSpec::Tag(
                                response.version.clone(),
                            )),
                        });
                    } else {
                        requested_packages.push(PackageSpec::Npm {
                            name: name.to_string(),
                            scope: scope.to_owned(),
                            requested: requested.to_owned(),
                        });
                    }
                }
            }

            tree.extend(response.tree);
        }

        bar.finish_and_clear();

        println!(
            "{} Resolved {} dependencies",
            format!("[{:.2}{}]", resolve_start.elapsed().as_secs_f32(), "s")
                .truecolor(156, 156, 156)
                .bold(),
            tree.len().to_string().truecolor(196, 206, 255).bold()
        );

        let install_start = Instant::now();

        let nm_dir = config.node_modules()?;
        let nm_volt_home = nm_dir.join(VoltConfig::VOLT_HOME);

        if !nm_dir.exists() {
            std::fs::create_dir_all(&nm_volt_home).unwrap();
        }

        let client = Client::builder().use_rustls_tls().build().unwrap();

        let mut incompatible_packages = vec![];

        // pnpm linking algorithm
        for value in tree.values() {
            // None means it's not platform-specific
            // We get a list of platforms, and if our current OS isn't on this list - it means that we can skip this package
            // this is only if the package is optional

            if let Some(os) = &value.os {
                if !os.contains(&"win32".to_string()) && !os.contains(&format!("!{}", "win32")) {
                    incompatible_packages.push(format!("{}@{}", value.name, value.version));
                    continue;
                }
            }

            if let Some(architecture) = &value.cpu {
                if !architecture.contains(&"x64".to_string()) {
                    incompatible_packages.push(format!("{}@{}", value.name, value.version));
                    continue;
                }
            }

            let mut name = value.name.clone();
            let mut scope: Option<String> = None;
            let mut last: Option<String> = None;

            if value.name.starts_with('@') {
                // replace @ with +
                name = name.replace('/', "+");

                scope = Some(
                    name.split('+')
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .first()
                        .unwrap()
                        .to_string(),
                );

                last = Some(
                    name.split('+')
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .last()
                        .unwrap()
                        .to_string(),
                );
            }

            std::fs::create_dir(nm_volt_home.join(format!("{}@{}", name, value.version)))
                .into_diagnostic()?;

            std::fs::create_dir(
                nm_volt_home
                    .join(format!("{}@{}", name, value.version))
                    .join("node_modules/"),
            )
            .into_diagnostic()?;

            if scope.is_none() {
                std::fs::create_dir(
                    nm_volt_home
                        .join(format!("{}@{}", name, value.version))
                        .join("node_modules/")
                        .join(&name),
                )
                .into_diagnostic()?;
            } else {
                std::fs::create_dir(
                    nm_volt_home
                        .join(format!("{}@{}", name, value.version))
                        .join("node_modules/")
                        .join(scope.as_ref().unwrap()),
                )
                .into_diagnostic()?;

                std::fs::create_dir(
                    nm_volt_home
                        .join(format!("{}@{}", name, value.version))
                        .join("node_modules/")
                        .join(scope.as_ref().unwrap())
                        .join(&last.unwrap()),
                )
                .into_diagnostic()?;
            }
        }

        for item in incompatible_packages {
            tree.remove(&item);
        }

        let total = tree.len();

        let bar = ProgressBar::new(total as u64);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .progress_chars("=>-"),
        );

        // todo: display progress bar for downloads that are taking time.
        tree.values()
            .map(|data| {
                install_package(
                    config.clone(),
                    data.clone(),
                    State {
                        http_client: client.clone(),
                    },
                )
            })
            .collect::<FuturesUnordered<_>>()
            .inspect(|_| bar.inc(1))
            .try_collect::<Vec<_>>()
            .await
            .unwrap();

        bar.finish_and_clear();

        // for package in requested_packages.iter() {
        //     if let PackageSpec::Npm {
        //         name,
        //         scope,
        //         requested,
        //     } = package
        //     {
        //         let mut node_modules_directory = config.node_modules().unwrap();

        //         // path to the package directory
        //         let mut package_directory = node_modules_directory
        //             .join(".volt")
        //             .join(format!("{}@{}", &name, requested.as_ref().unwrap()))
        //             .join("node_modules/")
        //             .join(&name);

        //         // path to the symlink
        //         let mut target_directory = node_modules_directory.join(name);

        //         #[cfg(windows)]
        //         junction::create(&package_directory, &target_directory).unwrap_or_else(|e| {
        //             eprintln!(
        //                 "target: {} destination: {}, {}",
        //                 package_directory.display(),
        //                 target_directory.display(),
        //                 e
        //             );
        //             std::process::exit(1);
        //         });

        //         #[cfg(unix)]
        //         std::os::unix::fs::symlink(package_directory, target_directory).unwrap_or_else(
        //             |e| {
        //                 eprintln!("{}", e);
        //                 std::process::exit(1);
        //             },
        //         );
        //     }
        // }

        println!(
            "{} Installed {} dependencies",
            format!("[{:.2}{}]", install_start.elapsed().as_secs_f32(), "s")
                .truecolor(156, 156, 156)
                .bold(),
            total.to_string().truecolor(196, 206, 255).bold()
        );

        // let (mut package_file, path) = PackageJson::get()?;

        // for package in requested_packages.iter() {
        //     package_file.add_dependency(package.to_owned());
        // }

        // // Save package.json
        // package_file.save()?;

        // // Save lockfiles
        // // global_lock_file.save()?;
        // // lock_file.save()?;

        Ok(())
    }
}
