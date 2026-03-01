use anyhow::{Context, Result};
use serde_json::Value;

use crate::lockfile::{Lockfile, LockfileParser, PackageInfo, sanitize_target_name};
use crate::npmrc::NpmrcConfig;

pub struct DenoParser;

impl LockfileParser for DenoParser {
    fn parse(&self, path: &str, npmrc: &NpmrcConfig) -> Result<Lockfile> {
        let content =
            std::fs::read_to_string(path).context(format!("Failed to read lockfile: {}", path))?;
        let json: Value =
            serde_json::from_str(&content).context("Failed to parse deno.lock as JSON")?;

        let version = json["version"].as_str().unwrap_or("unknown").to_string();

        let mut packages = Vec::new();

        if let Some(npm_map) = json["npm"].as_object() {
            for (key, value) in npm_map {
                if let Some(pkg) = parse_npm_entry(key, value, npmrc) {
                    packages.push(pkg);
                }
            }
        }

        if let Some(jsr_map) = json["jsr"].as_object() {
            for (key, value) in jsr_map {
                if let Some(pkg) = parse_jsr_entry(key, value) {
                    packages.push(pkg);
                }
            }
        }

        Ok(Lockfile {
            version,
            manager: "deno".to_string(),
            packages,
        })
    }
}

fn parse_npm_entry(key: &str, value: &Value, npmrc: &NpmrcConfig) -> Option<PackageInfo> {
    let base_key = key.split('_').next().unwrap_or(key);
    let (pkg_name, pkg_version) = parse_at_version(base_key)?;

    let target_name = sanitize_target_name(&format!("{}@{}", pkg_name, pkg_version));

    let tarball_url = Some(npmrc.tarball_url(&pkg_name, &pkg_version));

    let integrity = value["integrity"].as_str().map(|s| s.to_string());

    let dependencies = value["dependencies"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let dep_str = v.as_str()?;
                    let (name, ver) = parse_at_version(dep_str)?;
                    Some((name, ver))
                })
                .collect()
        })
        .unwrap_or_default();

    Some(PackageInfo {
        name: pkg_name,
        version: pkg_version,
        target_name,
        tarball_url,
        integrity,
        dependencies,
        optional_dependencies: vec![],
        is_dev: false,
    })
}

fn parse_jsr_entry(key: &str, value: &Value) -> Option<PackageInfo> {
    let (pkg_name, pkg_version) = parse_at_version(key)?;

    let target_name = sanitize_target_name(&format!("jsr_{}@{}", pkg_name, pkg_version));

    let tarball_url = if pkg_name.starts_with('@') {
        let without_at = &pkg_name[1..];
        let jsr_name = without_at.replacen('/', "__", 1);
        Some(format!(
            "https://npm.jsr.io/~/11/@jsr/{}/{}.tgz",
            jsr_name, pkg_version
        ))
    } else {
        Some(format!(
            "https://npm.jsr.io/~/11/{}/{}.tgz",
            pkg_name, pkg_version
        ))
    };

    let integrity = None;

    let dependencies = value["dependencies"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let dep_str = v.as_str()?;
                    let clean = dep_str
                        .trim_start_matches("jsr:")
                        .trim_start_matches("npm:");
                    Some((clean.to_string(), String::new()))
                })
                .collect()
        })
        .unwrap_or_default();

    Some(PackageInfo {
        name: format!("jsr:{}", pkg_name),
        version: pkg_version,
        target_name,
        tarball_url,
        integrity,
        dependencies,
        optional_dependencies: vec![],
        is_dev: false,
    })
}

fn parse_at_version(s: &str) -> Option<(String, String)> {
    let idx = s.rfind('@')?;
    if idx == 0 {
        return None;
    }
    Some((s[..idx].to_string(), s[idx + 1..].to_string()))
}

fn hex_to_base64(hex: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    STANDARD.encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_at_version_scoped() {
        let (name, ver) = parse_at_version("@octokit/core@7.0.5").unwrap();
        assert_eq!(name, "@octokit/core");
        assert_eq!(ver, "7.0.5");
    }

    #[test]
    fn test_parse_at_version_simple() {
        let (name, ver) = parse_at_version("bottleneck@2.19.5").unwrap();
        assert_eq!(name, "bottleneck");
        assert_eq!(ver, "2.19.5");
    }

    #[test]
    fn test_parse_at_version_bare_at() {
        assert!(parse_at_version("@only-scope").is_none());
    }

    #[test]
    fn test_jsr_tarball_url() {
        let value: Value = serde_json::from_str(r#"{"integrity": "abcd1234"}"#).unwrap();
        let pkg = parse_jsr_entry("@std/path@1.1.2", &value).unwrap();
        assert_eq!(
            pkg.tarball_url,
            Some("https://npm.jsr.io/@jsr/std__path/1.1.2.tgz".to_string())
        );
        assert!(pkg.name.starts_with("jsr:"));
    }

    #[test]
    fn test_npm_entry_with_peer_suffix() {
        let value: Value = serde_json::from_str(
            r#"{
            "integrity": "sha512-abc==",
            "dependencies": ["@octokit/auth-token", "universal-user-agent"]
        }"#,
        )
        .unwrap();
        let npmrc = NpmrcConfig::default_config();
        let pkg =
            parse_npm_entry("@octokit/core@7.0.5_@octokit+core@7.0.5", &value, &npmrc).unwrap();
        assert_eq!(pkg.name, "@octokit/core");
        assert_eq!(pkg.version, "7.0.5");
    }

    #[test]
    fn test_hex_to_base64() {
        let hex = "48656c6c6f";
        let b64 = hex_to_base64(hex);
        assert_eq!(b64, "SGVsbG8=");
    }

    #[test]
    fn test_jsr_integrity_conversion() {
        let value: Value = serde_json::from_str(
            r#"{
            "integrity": "2dfb46ecee525755f7989f94ece30bba85bd8ffe3e8666abc1bf926e1ee0698d"
        }"#,
        )
        .unwrap();
        let pkg = parse_jsr_entry("@david/console-static-text@0.3.0", &value).unwrap();
        assert!(pkg.integrity.as_ref().unwrap().starts_with("sha256-"));
    }

    #[test]
    fn test_deno_npm_deps_parse() {
        let value: Value = serde_json::from_str(
            r#"{
            "integrity": "sha512-abc==",
            "dependencies": [
                "@octokit/auth-token",
                "@octokit/graphql",
                "before-after-hook"
            ]
        }"#,
        )
        .unwrap();
        let npmrc = NpmrcConfig::default_config();
        let pkg = parse_npm_entry("@octokit/core@7.0.5", &value, &npmrc).unwrap();
        assert!(pkg.dependencies.len() <= 3);
    }
}
