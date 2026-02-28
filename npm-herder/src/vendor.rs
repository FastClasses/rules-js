use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use sha2::{Digest, Sha512};
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::PathBuf;
use tar::Archive;
use tempfile::NamedTempFile;

use crate::lockfile::PackageInfo;
use crate::npmrc::NpmrcConfig;

pub fn vendor_packages(
    packages: &[PackageInfo],
    vendor_dir: &str,
    npmrc: &NpmrcConfig,
) -> Result<()> {
    let vendor_path = PathBuf::from(vendor_dir);
    create_dir_all(&vendor_path)?;

    for pkg in packages {
        let out_dir = vendor_path.join(&pkg.target_name);
        if out_dir.exists() {
            println!("Skipping already vendored: {}", pkg.name);
            continue;
        }

        let tarball_url = match &pkg.tarball_url {
            Some(url) => url.clone(),
            None => {
                println!("Skipping {} (no resolution/tarball URL)", pkg.name);
                continue;
            }
        };

        println!(
            "Downloading {}@{} from {}",
            pkg.name, pkg.version, tarball_url
        );

        let token = npmrc.auth_token_for(&tarball_url);
        download_and_extract(&tarball_url, token, pkg.integrity.as_deref(), &out_dir)
            .with_context(|| format!("Failed to vendor {}@{}", pkg.name, pkg.version))?;
    }

    Ok(())
}

fn download_and_extract(
    url: &str,
    auth_token: Option<&str>,
    integrity: Option<&str>,
    out_dir: &PathBuf,
) -> Result<()> {
    let mut temp_file = NamedTempFile::new().context("Failed to create temp file")?;

    let client = Client::new();
    let mut request = client.get(url);
    if let Some(token) = auth_token {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        request = request.headers(headers);
    }

    let mut response = request
        .send()
        .context(format!("Download failed for {}", url))?
        .error_for_status()?;
    response.copy_to(temp_file.as_file_mut())?;

    if let Some(integrity_str) = integrity {
        verify_integrity(temp_file.path(), integrity_str)?;
    }
    let file = File::open(temp_file.path())?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    create_dir_all(out_dir)?;

    let file2 = File::open(temp_file.path())?;
    let decoder2 = GzDecoder::new(file2);
    let mut archive2 = Archive::new(decoder2);
    let prefix = detect_tar_prefix(&mut archive2);

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let path = entry.path()?.to_path_buf();

        let stripped = if let Some(ref pfx) = prefix {
            if path.starts_with(pfx) {
                path.strip_prefix(pfx).unwrap().to_path_buf()
            } else {
                path
            }
        } else {
            path
        };

        if stripped.as_os_str().is_empty() {
            continue;
        }

        let out_path = out_dir.join(&stripped);
        if let Some(parent) = out_path.parent() {
            create_dir_all(parent)?;
        }
        entry.unpack(&out_path)?;
    }

    Ok(())
}

fn verify_integrity(path: &std::path::Path, integrity: &str) -> Result<()> {
    if integrity.starts_with("sha512-") {
        let expected_b64 = integrity.trim_start_matches("sha512-");
        let expected = BASE64.decode(expected_b64)?;

        let mut hasher = Sha512::new();
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        hasher.update(&buf);
        let actual = hasher.finalize();

        if actual.as_slice() != expected.as_slice() {
            return Err(anyhow::anyhow!(
                "Integrity mismatch (sha512) for {}",
                path.display()
            ));
        }
    }

    Ok(())
}

fn detect_tar_prefix(archive: &mut Archive<GzDecoder<File>>) -> Option<PathBuf> {
    let mut prefix: Option<PathBuf> = None;

    for entry_result in archive.entries().ok()? {
        if let Ok(entry) = entry_result {
            if let Ok(path) = entry.path() {
                let first_component = path.components().next()?;
                let component_path = PathBuf::from(first_component.as_os_str());

                match &prefix {
                    None => prefix = Some(component_path),
                    Some(existing) => {
                        if *existing != component_path {
                            return None;
                        }
                    }
                }
            }
        }
    }

    prefix
}
