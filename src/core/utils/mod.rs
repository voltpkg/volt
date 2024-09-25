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

#[macro_use]
pub mod helper;
pub mod constants;
pub mod errors;
pub mod extensions;
pub mod package;
pub mod scripts;
pub mod voltapi;

use crate::{
    cli::VoltConfig,
    core::{io::extract_tarball, net::fetch_tarball, utils::voltapi::VoltPackage},
};

use errors::VoltError;
use futures::TryFutureExt;
use git_config::file::GitConfig;
use git_config::parser::parse_from_str;
use miette::{IntoDiagnostic, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use ssri::{Algorithm, Integrity};

use std::{collections::HashMap, fs::read_to_string, io::Write, path::PathBuf};

pub struct State {
    pub http_client: Client,
}

pub fn decompress_gzip(gz_data: &[u8]) -> Result<Vec<u8>> {
    // gzip RFC1952: a valid gzip file has an ISIZE field in the
    // footer, which is a little-endian u32 number representing the
    // decompressed size. This is ideal for libdeflate, which needs
    // preallocating the decompressed buffer.
    let isize = {
        let isize_start = gz_data.len() - 4;
        let isize_bytes: [u8; 4] = gz_data[isize_start..].try_into().into_diagnostic()?;
        u32::from_le_bytes(isize_bytes) as usize
    };

    let mut decompressor = libdeflater::Decompressor::new();

    let mut outbuf = vec![0; isize];
    decompressor
        .gzip_decompress(gz_data, &mut outbuf)
        .into_diagnostic()?;

    Ok(outbuf)
}

fn get_git_config_value_if_exists(
    config: &VoltConfig,
    section: &str,
    subsection: Option<&str>,
    key: &str,
) -> Result<Option<String>> {
    let config_path = config.home()?.join(".gitconfig");

    if config_path.exists() {
        let data = read_to_string(config_path).into_diagnostic()?;

        let parser = parse_from_str(&data).map_err(|err| VoltError::GitConfigParseError {
            error_text: err.to_string(),
        })?;

        let config = GitConfig::from(parser);
        let value = config.get_raw_value(section, subsection, key).ok();

        Ok(value.map(|v| String::from_utf8_lossy(&v).into_owned()))
    } else {
        Ok(None)
    }
}

/// Gets a config key from git using the git cli.
/// Uses `gitoxide` to read from your git configuration.
pub fn get_git_config(config: &VoltConfig, key: &str) -> Result<Option<String>> {
    match key {
        "user.name" => get_git_config_value_if_exists(config, "user", None, "name"),
        "user.email" => get_git_config_value_if_exists(config, "user", None, "email"),
        "repository.url" => get_git_config_value_if_exists(config, "remote", Some("origin"), "url"),
        _ => Ok(None),
    }
}

#[cfg(unix)]
pub fn enable_ansi_support() -> Result<(), u32> {
    Ok(())
}

// Windows Function
/// Enable ansi support and colors
#[cfg(windows)]
pub fn enable_ansi_support() -> Result<(), u32> {
    // ref: https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#EXAMPLE_OF_ENABLING_VIRTUAL_TERMINAL_PROCESSING @@ https://archive.is/L7wRJ#76%

    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::prelude::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::winnt::{FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};

    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    unsafe {
        // ref: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        // Using `CreateFileW("CONOUT$", ...)` to retrieve the console handle works correctly even if STDOUT and/or STDERR are redirected
        let console_out_name: Vec<u16> =
            OsStr::new("CONOUT$").encode_wide().chain(once(0)).collect();

        let console_handle = CreateFileW(
            console_out_name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );
        if console_handle == INVALID_HANDLE_VALUE {
            return Err(GetLastError());
        }

        // ref: https://docs.microsoft.com/en-us/windows/console/getconsolemode
        let mut console_mode: u32 = 0;
        if 0 == GetConsoleMode(console_handle, &mut console_mode) {
            return Err(GetLastError());
        }

        // VT processing not already enabled?
        if console_mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
            // https://docs.microsoft.com/en-us/windows/console/setconsolemode
            if 0 == SetConsoleMode(
                console_handle,
                console_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            ) {
                return Err(GetLastError());
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
/// Generates the binary and other required scripts for the package
pub fn generate_script(config: &VoltConfig, package: &VoltPackage) {
    use self::voltapi::Bin;

    let bin_path = config.node_modules().unwrap().join(".bin/");

    // // Create node_modules/scripts if it doesn't exist
    if !bin_path.exists() {
        // Create the binary directory
        std::fs::create_dir(&bin_path);
    }

    // Create binary scripts for the package if they exist.

    if package.bin.is_some() {
        let bin = package.bin.as_ref().unwrap();

        if let Bin::String(data) = bin {
        } else if let Bin::Map(map) = bin {
            let k = map.keys().next().unwrap();
            let v = map.values().next().unwrap();

            //             let cmd_file = format!(
            //                 r#"
            // @SETLOCAL
            // @IF NOT DEFINED NODE_PATH (
            //   @SET "NODE_PATH=PATHHERE"
            // ) ELSE (
            //   @SET "NODE_PATH=%NODE_PATH%;PATHHERE"
            // )
            // @IF EXIST "%~dp0\node.exe" (
            //   "%~dp0\node.exe"  "%~dp0\..\next\dist\bin\next" %*
            // ) ELSE (
            //   @SET PATHEXT=%PATHEXT:;.JS;=;%
            //   node  "%~dp0\..\next\dist\bin\next" %*
            // )"#,
            //             );

            //             let ps1_file = format!(
            //                 r#"#!/usr/bin/env pwsh
            // $basedir=Split-Path $MyInvocation.MyCommand.Definition -Parent

            // $exe=""
            // $pathsep=":"
            // $env_node_path=$env:NODE_PATH
            // $new_node_path="PATHHERE"
            // if ($PSVersionTable.PSVersion -lt "6.0" -or $IsWindows) {
            //   $exe=".exe"
            //   $pathsep=";"
            // } else {
            //   $new_node_path="PATHHERE"
            // }
            // if ([string]::IsNullOrEmpty($env_node_path)) {
            //   $env:NODE_PATH=$new_node_path
            // } else {
            //   $env:NODE_PATH="$env_node_path$pathsep$new_node_path"
            // }

            // $ret=0
            // if (Test-Path "$basedir/node$exe") {
            //   if ($MyInvocation.ExpectingInput) {
            //     $input | & "$basedir/node$exe"  "$basedir/../next/dist/bin/next" $args
            //   } else {
            //     & "$basedir/node$exe"  "$basedir/../next/dist/bin/next" $args
            //   }
            //   $ret=$LASTEXITCODE
            // } else {
            //   if ($MyInvocation.ExpectingInput) {
            //     $input | & "node$exe"  "$basedir/../next/dist/bin/next" $args
            //   } else {
            //     & "node$exe"  "$basedir/../next/dist/bin/next" $args
            //   }
            //   $ret=$LASTEXITCODE
            // }
            // $env:NODE_PATH=$env_node_path
            // exit $ret"#,
            //             );

            //             let executable_file = format!(
            //                 r#"#!/bin/sh
            // basedir=$(dirname "$(echo "$0" | sed -e 's,\\,/,g')")

            // case `uname` in
            //     *CYGWIN*) basedir=`cygpath -w "$basedir"`;;
            // esac

            // if [ -z "$NODE_PATH" ]; then
            //   export NODE_PATH="PATHHERE"
            // else
            //   export NODE_PATH="$NODE_PATH:PATHHERE"
            // fi
            // if [ -x "$basedir/node" ]; then
            //   exec "$basedir/node"  "$basedir/../next/dist/bin/next" "$@"
            // else
            //   exec node  "$basedir/../next/dist/bin/next" "$@"
            // fi"#,
            //             );

            //             let mut f = std::fs::File::create(format!(
            //                 r"{}/{}",
            //                 &bin_path.as_os_str().to_str().unwrap(),
            //                 k
            //             ))
            //             .unwrap();

            //             f.write_all(executable_file.as_bytes()).unwrap();

            //             let mut f = std::fs::File::create(format!(
            //                 r"{}/{}.cmd",
            //                 &bin_path.as_os_str().to_str().unwrap(),
            //                 k
            //             ))
            //             .unwrap();

            //             f.write_all(cmd_file.as_bytes()).unwrap();

            //             let mut f = std::fs::File::create(format!(
            //                 r"{}/{}.ps1",
            //                 &bin_path.as_os_str().to_str().unwrap(),
            //                 k
            //             ))
            //             .unwrap();

            //             f.write_all(ps1_file.as_bytes()).unwrap();
        }
    }
}

#[cfg(unix)]
// TODO: Put config second like everywhere else
pub fn generate_script(_config: &VoltConfig, _package: &VoltPackage) {
    // Create node_modules/scripts if it doesn't exist
    // if !Path::new("node_modules/scripts").exists() {
    //     std::fs::create_dir_all("node_modules/scripts").unwrap();
    // }

    // // If the package has binary scripts, create them
    // if package.bin.is_some() {
    //     let bin = package.bin.as_ref().unwrap();

    //     let k = bin.keys().next().unwrap();
    //     let v = bin.values().next().unwrap();

    //     let command = format!(
    //         r#"
    //         node  "{}/.volt/{}/{}" %*
    //         "#,
    //         app.volt_dir.to_string_lossy(),
    //         k,
    //         v,
    //     );
    //     // .replace(r"%~dp0\..", format!("{}", app.volt_dir.display()).as_str());
    //     let p = format!(r"node_modules/scripts/{}.sh", k);
    //     let mut f = File::create(p.clone()).unwrap();
    //     std::process::Command::new("chmod")
    //         .args(&["+x", &p])
    //         .spawn()
    //         .unwrap();
    //     f.write_all(command.as_bytes()).unwrap();
    // }
}

pub fn _check_peer_dependency(_package_name: &str) -> bool {
    false
}

pub fn verify_existing_installation(
    package: &VoltPackage,
    config: &VoltConfig,
) -> miette::Result<Vec<u8>> {
    let volt_home = config.volt_home()?;

    let result = cacache::read_sync(volt_home, package.cacache_key()).into_diagnostic()?;

    Ok(result)
}

pub fn verify_checksum(
    response: &bytes::Bytes,
    expected_checksum: &str,
) -> Result<(bool, Option<String>)> {
    // begin
    // there are only 2 supported algorithms
    // sha1 and sha512
    // so we can be sure that if it doesn't start with sha1, it's going to have to be sha512

    let algorithm = if expected_checksum.starts_with("sha1") {
        Algorithm::Sha1
    } else {
        Algorithm::Sha512
    };

    let calculated_checksum = VoltConfig::calc_hash(response, algorithm)?;

    if calculated_checksum == expected_checksum {
        Ok((true, None))
    } else {
        Ok((false, Some(calculated_checksum)))
    }
}

pub fn link_dependencies(package: &VoltPackage, config: &VoltConfig) -> miette::Result<()> {
    // link the subdependencies for a package
    if let Some(dependencies) = &package.dependencies {
        for (name, version) in dependencies.iter() {
            let name = name.replace(&format!("@{version}"), "");

            let mut dependency_link_path = config.node_modules()?;

            // node_modules/.volt
            dependency_link_path.push(".volt");

            // node_modules/.volt/accepts@1.2.3
            dependency_link_path.push(format!("{}@{}", name.replace('/', "+"), version));

            // node_modules/.volt/accepts@1.2.3/node_modules
            dependency_link_path.push("node_modules");

            // node_modules/.volt/accepts@1.2.3/node_modules/accepts
            dependency_link_path.push(&name);

            let mut target_link_path = config.node_modules()?;

            // node_modules/.volt
            target_link_path.push(".volt");

            // node_modules/.volt/accepts@1.2.3
            target_link_path.push(format!(
                "{}@{}",
                &package.name.replace('/', "+"),
                &package.version
            ));

            // node_modules/.volt/accepts@1.2.3/node_modules
            target_link_path.push("node_modules");

            // node_modules/.volt/accepts@1.2.3/node_modules/ms
            target_link_path.push(&name);

            #[cfg(windows)]
            junction::create(&dependency_link_path, &target_link_path).unwrap_or_else(|e| {
                eprintln!(
                    "target: {} destination: {}, {}",
                    target_link_path.display(),
                    dependency_link_path.display(),
                    e
                );
                std::process::exit(1);
            });

            #[cfg(unix)]
            std::os::unix::fs::symlink(dependency_link_path, target_link_path).unwrap_or_else(
                |e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                },
            );
        }
    }

    Ok(())
}

/// Install a JavaScript package.
pub async fn install_package(config: VoltConfig, package: VoltPackage, state: State) -> Result<()> {
    // Check if the package is already installed
    match verify_existing_installation(&package, &config) {
        Ok(value) => {
            let cas_file_map: Vec<(PathBuf, Integrity)> =
                serde_json::from_slice::<HashMap<PathBuf, Integrity>>(&value)
                    .unwrap()
                    .into_par_iter()
                    .map(|(k, v)| (k, v))
                    .collect();

            // Add package's directory to list of created directories
            let created_directories: Vec<PathBuf> = vec![];

            let mut package_path = config.node_modules()?;

            package_path.push(".volt/");
            package_path.push(format!("{}@{}", package.name, package.version));
            package_path.push("node_modules/");
            package_path.push(package.name.to_string().replace('/', r"\"));

            let mut handles = vec![];

            for chunk in cas_file_map.chunks(6) {
                let config_instance = config.clone();
                let package_path_instance = package_path.clone();
                let mut created_directories_instance = created_directories.clone();

                let chunk_instance = chunk.to_vec();

                handles.push(tokio::task::spawn_blocking(move || {
                    for (name, hash) in chunk_instance.clone() {
                        let contents =
                            cacache::read_hash_sync(config_instance.clone().volt_home()?, &hash)
                                .into_diagnostic()?;

                        let file_path = package_path_instance.clone().join(&name);

                        // If we haven't created this directory yet, create it
                        if !created_directories_instance
                            .clone()
                            .iter()
                            .any(|p| p == &file_path)
                        {
                            if let Some(value) = name.parent() {
                                created_directories_instance.push(file_path.to_path_buf());
                                std::fs::create_dir_all(&package_path_instance.join(value))
                                    .into_diagnostic()?;
                            }
                        }

                        // Write the contents to node_modules
                        let mut file = std::fs::File::create(&file_path).unwrap();

                        file.write_all(&contents).into_diagnostic()?;
                    }

                    Ok(()) as Result<()>
                }));
            }

            for handle in handles {
                handle
                    .unwrap_or_else(|e| {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    })
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    });
            }

            link_dependencies(&package, &config)?;
        }
        Err(_) => {
            // fetch the tarball from the registry
            let response = fetch_tarball(&package, state).await?;

            tokio::task::spawn_blocking({
                let config = config.clone();
                let package = package.clone();
                move || -> Result<()> {
                    // verify the checksum
                    // (checksum is valid, calculated checksum)
                    let (verified, _checksum) = verify_checksum(&response, &package.integrity)?;

                    if verified {
                        // decompress gzipped response
                        let decompressed_response = decompress_gzip(&response)?;

                        // extract the tarball
                        extract_tarball(decompressed_response, &package, &config)?;

                        // generate .bin files
                        generate_script(&config, &package);

                        // generate symlinks
                        link_dependencies(&package, &config)?;
                    } else {
                        // TODO: handle checksum failure
                    }

                    Ok(())
                }
            })
            .await
            .into_diagnostic()??;
        }
    }

    Ok(())
}
