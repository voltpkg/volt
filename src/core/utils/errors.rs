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

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum VoltError {
    // #[error("failed to enable ansi support")]
    // #[diagnostic(code(volt::environment::enable_ansi_support))]
    // EnableAnsiSupport(),
    #[error("failed to detect `{env}`")]
    #[diagnostic(code(volt::environment::get))]
    EnvironmentError { source: std::io::Error, env: String },

    #[error("failed to parse package specification: `{spec}`")]
    #[diagnostic(code(volt::package_spec::parse))]
    _PackageSpecificationError { spec: String },

    #[error("failed to detect your home directory")]
    #[diagnostic(code(volt::environment::home_dir))]
    GetHomeDirError,

    #[error("failed to get the name of the current directory")]
    #[diagnostic(code(volt::environment::current_dir_name))]
    GetCurrentDirNameError,

    // #[error("failed to initialize lz4 decoder")]
    // #[diagnostic(code(volt::decode::lz4::initialize))]
    // DecoderError(#[source] std::io::Error),

    // #[error("failed to decode lz4 encoded data")]
    // #[diagnostic(code(volt::decode::lz4::decode))]
    // DecodeError(#[source] std::io::Error),
    #[error("failed to receive response from the registry")]
    #[diagnostic(code(volt::network))]
    NetworkError(isahc::Error),

    // #[error("failed to receive byte response")]
    // #[diagnostic(code(volt::network::rec))]
    // NetworkRecError(#[source] std::io::Error),
    #[error("failed to create directory")]
    #[diagnostic(code(volt::io::create_dir))]
    _CreateDirError(#[source] std::io::Error),

    #[error("GET {url} - 404 - {package_name} was not found in the volt registry, or you don't have the permission to request it.")]
    #[diagnostic(code(volt::registry::volt::package_not_found))]
    PackageNotFound { url: String, package_name: String },

    #[error("GET {url} - 429 - Too many requests has been sent to {url} on the volt registry. Please try again later.")]
    #[diagnostic(code(volt::registry::volt::too_many_requests))]
    TooManyRequests { url: String },

    #[error("GET {url} - 400 - Bad request. Please try again later.")]
    #[diagnostic(code(volt::registry::volt::bad_request))]
    BadRequest { url: String },

    #[error("GET {url} - {} - An unknown error occurred. Please try again later.")]
    #[diagnostic(code(volt::registry::volt::unknown_error))]
    NetworkUnknownError {
        url: String,
        package_name: String,
        code: String,
    },

    #[error("failed to parse {hash} integrity hash.")]
    #[diagnostic(code(volt::integrity::parse))]
    _HashParseError { hash: String },

    #[error("failed to copy bytes to hasher.")]
    #[diagnostic(code(volt::hasher::copy))]
    _HasherCopyError(#[source] std::io::Error),

    #[error("failed to verify tarball checksum")]
    #[diagnostic(code(volt::integrity::verify))]
    _ChecksumVerificationError,

    #[error("failed to convert integrity into hex")]
    #[diagnostic(code(volt::integrity::convert))]
    _IntegrityConversionError,

    #[error("failed to deserialize slice to `SpeedyVoltResponse`")]
    #[diagnostic(code(volt::integrity::convert))]
    _DeserializeError,

    #[error("failed to build request client")]
    #[diagnostic(code(volt::network::builder))]
    _RequestBuilderError(#[source] isahc::http::Error),

    #[error("failed to build receive response text")]
    #[diagnostic(code(volt::io::rec::text))]
    _IoTextRecError(#[source] std::io::Error),

    #[error("failed to find a hash that matches the specified version requirement: {version}")]
    #[diagnostic(code(volt::io::rec::text))]
    _HashLookupError { version: String },

    #[error("failed to find a version that matches the specified version requirement for {name}")]
    #[diagnostic(code(volt::io::rec::text))]
    _VersionLookupError { name: String },

    #[error("failed to read `{name}`")]
    #[diagnostic(code(volt::io::file::read))]
    ReadFileError {
        source: std::io::Error,
        name: String,
    },

    #[error("failed to write to `{name}`")]
    #[diagnostic(code(volt::io::file::write))]
    WriteFileError {
        source: std::io::Error,
        name: String,
    },

    // Convert error to `String` instead of having a `source` because `git_config::parser::Error`
    // has a lifetime parameter
    #[error("failed to parse git configuration file: `{error_text}`")]
    #[diagnostic(code(volt::git::parse))]
    GitConfigParseError { error_text: String },

    #[error("an unknown error occurred.")]
    #[diagnostic(code(volt::unknown))]
    _UnknownError,
}
