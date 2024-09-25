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

//! Handle an unknown command (can be listed in scripts).

use miette::Result;

use async_trait::async_trait;

use crate::cli::{VoltCommand, VoltConfig};
// use colored::Colorize;
// use indicatif::ProgressBar;
// use indicatif::ProgressStyle;
// use walkdir::WalkDir;

// use syntect::easy::HighlightLines;
// use syntect::highlighting::{Style, ThemeSet};
// use syntect::parsing::SyntaxSet;
// use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

// const PROGRESS_CHARS: &str = "=> ";
pub struct Watch {}

// fn gen_pointer_string(start: u128, end: u128) -> String {
//     if start != end {
//         let range = end - start;
//         return format!("{}", "^".repeat(range as usize));
//     } else {
//         return format!("{}^", (" ".repeat((start - 1) as usize)));
//     }
// }

// fn src_folder_exists() -> bool {
//     for f in read_dir(".").unwrap() {
//         let f = f.unwrap();
//         if f.path().is_dir() && format!("{}", f.path().display()).as_str() == r".\src" {
//             return true;
//         }
//     }
//     false
// }

// fn get_top_elements(elements: &[String]) -> Vec<String> {
//     let mut file_names: Vec<String> = vec![];
//     let vec = elements[..3.min(elements.len())].to_vec();

//     for item in &vec {
//         let split = item.split(r"\").collect::<Vec<&str>>();
//         let final_element = split.last().unwrap();
//         file_names.push(final_element.to_string());
//     }

//     file_names
// }

#[async_trait]
impl VoltCommand for Watch {
    /// Execute the `volt watch` command
    ///
    /// Execute a watch command
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// //
    /// // .exec() is an async call so you need to await it
    /// Watch.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        // // Set current dir
        // let mut current_dir = std::env::current_dir().unwrap();

        // // Set list for all JS files
        // let mut files: Vec<String> = vec![];

        // // Scan for all JS files
        // // Code must be in src folder
        // if !current_dir.ends_with("src") && src_folder_exists() {
        //     current_dir = current_dir.join("src");
        // }

        // if current_dir.ends_with("src") {
        //     for entry in WalkDir::new(current_dir) {
        //         let entry = entry.unwrap();
        //         let file_name = format!("{}", entry.path().display());
        //         if file_name.ends_with(".js")
        //             || file_name.ends_with(".ts")
        //             || file_name.ends_with(".json")
        //         {
        //             files.push(file_name);
        //         }
        //     }
        // }

        // if files.len() > 0 {
        //     let progress_bar = ProgressBar::new(files.len() as u64);

        //     progress_bar.set_style(
        //         ProgressStyle::default_bar()
        //             .progress_chars(PROGRESS_CHARS)
        //             .template(&format!(
        //                 "{} [{{bar:20.magenta/blue}}] {{pos}} / {{len}} {{msg:.yellow}}",
        //                 "Scanning Code".bright_cyan()
        //             )),
        //     );

        //     let mut files_message_vec = files.clone();
        //     let mut modules: Vec<String> = vec![];
        //     println!("press c to copy error, type ctrl+g to look up on google, type ctrl+o to look up on stackoverflow");
        //     for f in files {
        //         // display next 3 files to be analyzed
        //         let file_names;
        //         file_names = get_top_elements(&files_message_vec.as_slice());

        //         let message = file_names.join(", ");
        //         progress_bar.set_message(message);

        //         let mut syntax = Syntax::default();

        //         if f.clone().ends_with(".ts") {
        //             syntax = syntax.typescript();
        //         }

        //         let text = read_to_string(&f).unwrap();
        //         let res = rslint_parser::parse_with_syntax(&text.as_str(), 0, syntax);

        //         let errors = res.errors();

        //         if errors != [] {
        //             progress_bar.finish_and_clear();
        //             for err in errors {
        //                 let ps = SyntaxSet::load_defaults_newlines();
        //                 let ts = ThemeSet::get_theme("dracula.tmTheme").unwrap();
        //                 let file_name = Path::new(&f).file_name().unwrap().to_str().unwrap();
        //                 let code = &err
        //                     .code
        //                     .as_ref()
        //                     .unwrap()
        //                     .to_lowercase()
        //                     .replace("error", "");
        //                 let _severity = &err.severity;
        //                 let title = &err.title;

        //                 let start = *&err.primary.as_ref().unwrap().span.range.start as u128;
        //                 let end = *&err.primary.as_ref().unwrap().span.range.end as u128;
        //                 let chars = &text.chars().collect::<Vec<char>>()[0..(start - 1) as usize];
        //                 let line_number = chars.iter().filter(|&n| *n == '\n').count() + 1;
        //                 let line_error = text.lines().collect::<Vec<&str>>()[line_number - 1];

        //                 let syntax = ps.find_syntax_by_extension("js").unwrap();
        //                 let mut h = HighlightLines::new(syntax, &ts);
        //                 let mut colorized_line = String::new();

        //                 for line in LinesWithEndings::from(line_error) {
        //                     let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        //                     colorized_line = as_24_bit_terminal_escaped(&ranges[..], false);
        //                 }

        //                 println!("{} {}", &start, &end);

        //                 println!(" {} {}:{}", "-->".bright_black(), &file_name, line_number);
        //                 println!("  {}", "|".bright_black());
        //                 println!(
        //                     "{}{} {}",
        //                     line_number.to_string().bright_black(),
        //                     " |".bright_black(),
        //                     colorized_line
        //                 );
        //                 println!(
        //                     "  {}{}",
        //                     "|".bright_black(),
        //                     gen_pointer_string(start, end).bright_green()
        //                 );
        //                 println!("  {}", "|".bright_black());
        //                 termimad::print_text(
        //                     format!("error({}): {}", code.bright_yellow(), title).as_str(),
        //                 );

        //                 std::process::exit(0);
        //             }
        //         }

        //         let require_regex = Regex::new(
        //             r#"(const|let|var) [a-zA-Z0-9_]+\s?=\s?require\s?\(('|")(.*)('|")\)\s?;?"#,
        //         )
        //         .unwrap();

        //         for cap in require_regex.captures_iter(&text.as_str()) {
        //             let element = cap.get(3).unwrap().as_str().to_string();

        //             if !modules.contains(&element) {
        //                 modules.push(element);
        //             }
        //         }

        //         let import_regex =
        //             Regex::new(r#"import\s+(\{?).*(\}?)(\s?)from(\s?)('|")(.*)('|");?"#).unwrap();

        //         for cap in import_regex.captures_iter(&text.as_str()) {
        //             let element = cap.get(6).unwrap().as_str().to_string();
        //             if !modules.contains(&element) {
        //                 modules.push(element);
        //             }
        //         }

        //         files_message_vec.remove(0);
        //         progress_bar.inc(1);
        //     }
        //     progress_bar.finish_with_message("");

        // let node_modules = read_dir("node_modules").unwrap();
        // let mut available_modules: Vec<String> = vec![];

        // for entry in node_modules {
        //     let entry = entry.unwrap();
        //     if entry.path().is_dir() {
        //         let path = entry.file_name().to_str().unwrap().to_string();
        //         available_modules.push(path);
        //     }
        // }

        // let mut missing_modules: Vec<String> = vec![];

        // for module in modules {
        //     if !available_modules.contains(&module) {
        //         missing_modules.push(module);
        //     }
        // }

        // Set list of modules which are not found
        // let mut modules: Vec<String> = vec![];

        // for file in files {
        //     let file_split: Vec<&str> = file.split(r"\").collect();
        //     let file_name = file_split[file_split.len() - 1];
        //     let output = process::Command::new("node").arg(file_name).output()?;
        //     let code = output.status.code().unwrap();
        //     if code == 1 {
        //         let err_message = String::from_utf8(output.stderr)?;
        //         let re = Regex::new(r"Cannot find module '(.+)'").unwrap();
        //         let matches: Vec<&str> = re
        //             .captures_iter(&err_message)
        //             .map(|c| c.get(1).unwrap().as_str())
        //             .collect();
        //         for _match in matches {
        //             modules.push(_match.to_string());
        //         }
        //     }
        // }

        // Set args for adding packages
        // let mut args: Vec<String> = vec!["add".to_string()];
        // if modules.len() > 0 {
        //     println!("Found missing modules.\nPress {} to select the modules and {} to install the selected modules", "space".bright_cyan(), "enter".bright_cyan());
        //     let chosen_modules: Vec<usize> = MultiSelect::new().items(&modules).interact()?;
        //     for chosen_module in chosen_modules {
        //         let module = &modules[chosen_module];
        //         args.push(module.to_string());
        //     }
        // }

        // Initialize app
        // let mut app = App::initialize();

        // Set the args for the app
        // app.args = args.clone();

        // if &args.len() > &1 {
        //     // Add the modules
        //     add::command::Add::exec(Arc::new(app)).await.unwrap();
        // }
        // }

        Ok(())
    }
}
