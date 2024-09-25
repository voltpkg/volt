use std::time::Instant;

use crate::core::{
    utils::constants::MAX_RETRIES,
    utils::errors::VoltError,
    utils::voltapi::{VoltPackage, VoltResponse},
    utils::State,
};

use colored::Colorize;
use futures_util::{stream::FuturesUnordered, StreamExt};
use indicatif::ProgressBar;
use isahc::AsyncReadResponseExt;
use miette::{IntoDiagnostic, Result};
use package_spec::PackageSpec;
use reqwest::StatusCode;
use speedy::Readable;

pub async fn get_volt_response_multi(
    packages: &[PackageSpec],
    progress_bar: &ProgressBar,
) -> Vec<Result<VoltResponse>> {
    packages
        .iter()
        .map(|spec| {
            if let PackageSpec::Npm {
                name, requested, ..
            } = spec
            {
                let mut version: String = "latest".to_string();

                if requested.is_some() {
                    version = requested.as_ref().unwrap().to_string();
                };

                progress_bar.set_message(format!("{}@{}", name, version.truecolor(125, 125, 125)));
            }

            get_volt_response(spec)
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<VoltResponse>>>()
        .await
}

// Get response from volt CDN
pub async fn get_volt_response(package_spec: &PackageSpec) -> Result<VoltResponse> {
    // number of retries
    let mut retries = 0;

    // we know that PackageSpec is of type npm (we filtered the non-npm ones out)

    if let PackageSpec::Npm { name, .. } = package_spec {
        // loop until MAX_RETRIES reached.
        loop {
            // get a response
            let mut response =
                isahc::get_async(format!("http://registry.voltpkg.com/{}.sp", &package_spec))
                    .await
                    .map_err(VoltError::NetworkError)?;

            // check the status of the response
            match response.status() {
                // 200 (OK)
                StatusCode::OK => {
                    let mut response: VoltResponse =
                        VoltResponse::read_from_buffer(&response.bytes().await.unwrap()).unwrap();

                    response.name = name.to_string();

                    return Ok(response);
                }
                // 429 (TOO_MANY_REQUESTS)
                StatusCode::TOO_MANY_REQUESTS => {
                    return Err(VoltError::TooManyRequests {
                        url: format!("http://registry.voltpkg.com/{}.sp", &package_spec),
                    }
                    .into());
                }
                // 400 (BAD_REQUEST)
                StatusCode::BAD_REQUEST => {
                    return Err(VoltError::BadRequest {
                        url: format!("http://registry.voltpkg.com/{}.sp", &package_spec),
                    }
                    .into());
                }
                // 404 (NOT_FOUND)
                StatusCode::NOT_FOUND if retries == MAX_RETRIES => {
                    return Err(VoltError::PackageNotFound {
                        url: format!("http://registry.voltpkg.com/{}.sp", &package_spec),
                        package_name: package_spec.to_string(),
                    }
                    .into());
                }
                // Other Errors
                _ => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::NetworkUnknownError {
                            url: format!("http://registry.voltpkg.com/{}.sp", name),
                            package_name: package_spec.to_string(),
                            code: response.status().as_str().to_string(),
                        }
                        .into());
                    }
                }
            }

            retries += 1;
        }
    } else {
        panic!("Volt does not support non-npm package specifications yet.");
    }
}

/// downloads and extracts tarball file from package
pub async fn fetch_tarball(package: &VoltPackage, state: State) -> Result<bytes::Bytes> {
    // Receive the tarball from the npm registry
    let response = state
        .http_client
        .get(&package.tarball)
        .send()
        .await
        .into_diagnostic()?
        .bytes()
        .await
        .into_diagnostic()?;

    Ok(response)
}

pub async fn fetch_dep_tree(
    data: &[PackageSpec],
    progress_bar: &ProgressBar,
) -> Result<Vec<VoltResponse>> {
    if data.len() > 1 {
        Ok(get_volt_response_multi(data, progress_bar)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?)
    } else {
        if let PackageSpec::Npm {
            name, requested, ..
        } = &data[0]
        {
            let mut version: String = "latest".to_string();

            if requested.is_some() {
                version = requested.as_ref().unwrap().to_string();
            };

            progress_bar.set_message(format!("{}@{}", name, version.truecolor(125, 125, 125)));
        }

        Ok(vec![get_volt_response(&data[0]).await?])
    }
}

pub async fn _ping() {
    let _ping = Instant::now();

    println!("PING! http://registry.voltpkg.com/");

    let response = isahc::get_async("http://registry.voltpkg.com/ping")
        .await
        .unwrap();

    match response.status() {
        StatusCode::OK => {
            let pong = Instant::now();

            println!(
                "PONG! http://registry.voltpkg.com/ {}",
                pong.elapsed().as_secs_f32()
            );
        }
        _ => {
            println!("Ping failed");
        }
    }

    let _ping = Instant::now();

    println!("PING! https://registry.npmjs.org/");

    let response = isahc::get_async("https://registry.npmjs.org/")
        .await
        .unwrap();

    match response.status() {
        StatusCode::OK => {
            let pong = Instant::now();

            println!(
                "PONG! https://registry.npmjs.org/ {}",
                pong.elapsed().as_secs_f32()
            );
        }
        _ => {
            println!("Ping failed");
        }
    }
}
