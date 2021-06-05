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

//! Installs dependencies for a project.

// Std Imports
use std::sync::Arc;

// Library Imports
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;

// Crate Level Imports
use crate::VERSION;
use crate::{classes::package::PackageJson, utils::App};

// Super Imports
use super::{add::Add, Command};

/// Struct implementation for the `Install` command.
pub struct Install;

#[async_trait]
impl Command for Install {
    /// Display a help menu for the `volt install` command.
    fn help() -> String {
        format!(
            r#"volt {}
        
Install dependencies for a project.

Usage: {} {} {}
    
Options: 
    
  {} {} Accept all prompts while installing dependencies.  
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "install".bright_purple(),
            "[flags]".white(),
            "--yes".blue(),
            "(-y)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt install` command
    ///
    /// Install dependencies for a project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Install dependencies for a project with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Install.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        let package_file = PackageJson::from("package.json");
        let dependencies = package_file.dependencies;

        let mut app = App::initialize();

        app.args = dependencies.into_iter().map(|value| value.0).collect();

        Add::exec(Arc::new(app)).await.unwrap();

        Ok(())
    }
}
