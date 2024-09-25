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

//! Clean cached download files.

use crate::{
    App,
    Command,
    coreutils::constants::PROGRESS_CHARS,
};
use volt_core::VERSION;

use async_trait::async_trait;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;

use std::{
    env::temp_dir,
    fs::remove_file,
    process::exit,
    sync::Arc,
};

/// Struct implementation for the `Add` command.
#[derive(Clone)]
pub struct Cache {}

#[async_trait]
impl Command for Cache {
    /// Display a help menu for the `volt cache` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Handle the volt cache files.
Usage: {} {} {}

Commands:
  clean - Clean downloaded cache files and metadata. 

Options: 
    
  {} {} Output verbose messages on internal operations.
  {} {} Disable progress bar."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "cache".bright_purple(),
            "[command]".bright_purple(),
            "--verbose".blue(),
            "(-v)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    /// Execute the `volt cache` command
    ///
    /// Clean your download cache.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Clean your download cache (does not break symlinks)
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        // if app.args.len() == 1 {
        //     println!("{}", Self::help());
        //     exit(1);
        // }
        // if app.args[1].as_str() == "clean" {
        //     let files: Vec<_> = fs::read_dir(temp_dir().join("volt")).unwrap().collect();

        //     let count = files.len();

        //     let progress_bar = ProgressBar::new(count.to_owned() as u64);

        //     progress_bar.set_style(
        //         ProgressStyle::default_bar()
        //             .progress_chars(PROGRESS_CHARS)
        //             .template(&format!(
        //                 "{} [{{bar:40.magenta/blue}}] {{msg:.blue}} {{len}} / {{pos}}",
        //                 "Deleting Cache".bright_blue()
        //             )),
        //     );

        //     for file in files {
        //         let os_str = file.unwrap().file_name();
        //         let f_name = format!(r"{}volt\{}", temp_dir().display(), os_str.to_str().unwrap());

        //         remove_file(f_name).unwrap();
        //         progress_bar.inc(1);
        //     }

        //     progress_bar.finish();
        // }
        Ok(())
    }
}
