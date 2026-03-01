use anyhow::{Context, Result};
use yaml_serde::Value;

use crate::lockfile::{Lockfile, LockfileParser, PackageInfo, sanitize_target_name};
use crate::npmrc::NpmrcConfig;

pub struct BunParser;

impl LockfileParser for BunParser {
    fn parse(&self, path: &str, npmrc: &NpmrcConfig) -> Result<Lockfile> {
        let content =
            std::fs::read_to_string(path).context(format!("Failed to read lockfile: {}", path))?;
        let json: Value = yaml_serde::from_str(&content)
            .context("Failed to parse bun.lock as JSONC using yaml_serde")?;

        let version = json["lockfileVersion"]
            .as_i64()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "1".to_string());

        let dev_deps = extract_dev_deps(&json);
        let packages = extract_packages(&json, npmrc, &dev_deps)?;

        Ok(Lockfile {
            version,
            manager: "bun".to_string(),
            packages,
        })
    }
}

fn extract_dev_deps(json: &Value) -> std::collections::HashSet<String> {
    let mut devs = std::collections::HashSet::new();
    if let Some(workspaces) = json["workspaces"].as_mapping() {
        for (_ws_key, ws_val) in workspaces {
            if let Some(dd) = ws_val["devDependencies"].as_mapping() {
                for (name, _) in dd {
                    if let Some(n) = name.as_str() {
                        devs.insert(n.to_string());
                    }
                }
            }
        }
    }
    devs
}

fn extract_packages(
    json: &Value,
    npmrc: &NpmrcConfig,
    dev_deps: &std::collections::HashSet<String>,
) -> Result<Vec<PackageInfo>> {
    let packages = match json["packages"].as_mapping() {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();

    for (key, value) in packages {
        let tuple = match value.as_sequence() {
            Some(t) if !t.is_empty() => t,
            _ => continue,
        };

        let resolved = tuple[0].as_str().unwrap_or_default();

        if resolved.is_empty() {
            continue;
        }

        let (pkg_name, pkg_version) = parse_bun_resolved(resolved);
        if pkg_name.is_empty() || pkg_version.starts_with("workspace:") {
            continue;
        }

        let target_name = sanitize_target_name(&format!("{}@{}", pkg_name, pkg_version));

        let tarball_raw = tuple.get(1).and_then(|v| v.as_str()).unwrap_or_default();
        let tarball_url = if tarball_raw.is_empty() {
            Some(npmrc.tarball_url(&pkg_name, &pkg_version))
        } else if tarball_raw.starts_with("http") {
            Some(tarball_raw.to_string())
        } else {
            // GitHub or other specifier — skip vendoring
            None
        };

        let meta = tuple.get(2).unwrap_or(&Value::Null);
        let dependencies = extract_dep_map(&meta["dependencies"]);
        let optional_dependencies = extract_dep_map(&meta["optionalDependencies"]);

        let integrity = tuple.get(3).and_then(|v| v.as_str()).map(|s| s.to_string());

        let is_dev = dev_deps.contains(&pkg_name);

        result.push(PackageInfo {
            name: pkg_name,
            version: pkg_version,
            target_name,
            tarball_url,
            integrity,
            dependencies,
            optional_dependencies,
            is_dev,
        });
    }

    Ok(result)
}

fn parse_bun_resolved(resolved: &str) -> (String, String) {
    let sep = if resolved.starts_with('@') {
        resolved[1..].find('@').map(|i| i + 1)
    } else {
        resolved.find('@')
    };

    match sep {
        Some(idx) => (resolved[..idx].to_string(), resolved[idx + 1..].to_string()),
        _ => (resolved.to_string(), String::new()),
    }
}

fn extract_dep_map(value: &Value) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let Some(obj) = value.as_mapping() {
        for (name, ver) in obj {
            if let Some(n) = name.as_str() {
                let v = ver.as_str().unwrap_or_default().to_string();
                result.push((n.to_string(), v));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bun_resolved_scoped() {
        let (name, ver) = parse_bun_resolved("@sveltejs/kit@2.5.0");
        assert_eq!(name, "@sveltejs/kit");
        assert_eq!(ver, "2.5.0");
    }

    #[test]
    fn test_parse_bun_resolved_simple() {
        let (name, ver) = parse_bun_resolved("rollup@4.59.0");
        assert_eq!(name, "rollup");
        assert_eq!(ver, "4.59.0");
    }

    #[test]
    fn test_extract_dev_deps() {
        let json: Value = yaml_serde::from_str(
            r#"{
            "workspaces": {
                "": {
                    "devDependencies": {
                        "typescript": "^5.0",
                        "prettier": "^3.0"
                    }
                }
            }
        }"#,
        )
        .unwrap();
        let devs = extract_dev_deps(&json);
        assert!(devs.contains("typescript"));
        assert!(devs.contains("prettier"));
        assert!(!devs.contains("react"));
    }

    #[test]
    fn test_extract_packages_basic() {
        let json: Value = yaml_serde::from_str(r#"{
            "workspaces": {},
            "packages": {
                "react": ["react@18.3.1", "", {"dependencies": {"loose-envify": "^1.1.0"}}, "sha512-abc123=="]
            }
        }"#).unwrap();
        let npmrc = NpmrcConfig::default_config();
        let devs = std::collections::HashSet::new();
        let pkgs = extract_packages(&json, &npmrc, &devs).unwrap();
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "react");
        assert_eq!(pkgs[0].version, "18.3.1");
        assert_eq!(pkgs[0].integrity, Some("sha512-abc123==".to_string()));
        assert_eq!(pkgs[0].dependencies.len(), 1);
    }

    #[test]
    fn test_skip_workspace_packages() {
        let json: Value = yaml_serde::from_str(
            r#"{
            "workspaces": {},
            "packages": {
                "@types/bun": ["@types/bun@workspace:packages/@types/bun"]
            }
        }"#,
        )
        .unwrap();
        let npmrc = NpmrcConfig::default_config();
        let devs = std::collections::HashSet::new();
        let pkgs = extract_packages(&json, &npmrc, &devs).unwrap();
        assert_eq!(pkgs.len(), 0);
    }
}
