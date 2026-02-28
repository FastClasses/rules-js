use anyhow::{Context, Result};
use yaml_serde::Value;

use crate::lockfile::{sanitize_target_name, Lockfile, LockfileParser, PackageInfo};
use crate::npmrc::NpmrcConfig;

pub struct PnpmParser;

impl LockfileParser for PnpmParser {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn parse(&self, path: &str, npmrc: &NpmrcConfig) -> Result<Lockfile> {
        let content =
            std::fs::read_to_string(path).context(format!("Failed to read lockfile: {}", path))?;
        let yaml: Value =
            yaml_serde::from_str(&content).context("Failed to parse lockfile YAML")?;

        let version = yaml["lockfileVersion"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let major: f64 = version.parse().unwrap_or(0.0);
        if major < 6.0 {
            eprintln!(
                "Warning: pnpm lockfile version {} is older than expected (6.0+).",
                version
            );
        } else if major > 10.0 {
            eprintln!(
                "Warning: pnpm lockfile version {} is newer than supported (10.0).",
                version
            );
        }

        let has_snapshots = yaml["snapshots"].as_mapping().is_some();
        let packages = if has_snapshots {
            extract_from_snapshots(&yaml, npmrc)?
        } else {
            extract_from_packages(&yaml, npmrc)?
        };

        Ok(Lockfile {
            version,
            manager: "pnpm".to_string(),
            packages,
        })
    }
}

fn parse_dep_path(dep_path: &str) -> (&str, &str) {
    let path = dep_path.trim_start_matches('/');
    let sep_index = path.rfind('@').unwrap_or(0);

    if sep_index == 0 {
        return (path, "");
    }

    (&path[..sep_index], &path[sep_index + 1..])
}

fn remove_peer_suffix(dep_path: &str) -> &str {
    dep_path.split('(').next().unwrap_or(dep_path)
}

fn extract_from_snapshots(yaml: &Value, npmrc: &NpmrcConfig) -> Result<Vec<PackageInfo>> {
    let snapshots = yaml["snapshots"]
        .as_mapping()
        .context("No 'snapshots' section in lockfile")?;
    let packages_meta = yaml["packages"].as_mapping();

    let mut result = Vec::new();
    let mut seen_base_keys = std::collections::HashSet::new();

    for (snap_k, snap_v) in snapshots {
        let snap_key = snap_k.as_str().unwrap_or_default();
        if snap_key.is_empty() {
            continue;
        }

        let base_key = remove_peer_suffix(snap_key);

        if !seen_base_keys.insert(base_key.to_string()) {
            continue;
        }

        let (pkg_name, pkg_version) = parse_dep_path(base_key);
        let target_name = sanitize_target_name(base_key);

        let pkg_meta =
            packages_meta.and_then(|pkgs| pkgs.get(&Value::String(base_key.to_string())));

        let tarball_url = pkg_meta
            .and_then(|m| m["resolution"]["tarball"].as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                if !pkg_name.is_empty() {
                    Some(npmrc.tarball_url(pkg_name, pkg_version))
                } else {
                    None
                }
            });

        let integrity = pkg_meta
            .and_then(|m| m["resolution"]["integrity"].as_str())
            .map(|s| s.to_string());
        let dependencies = extract_dep_map(&snap_v["dependencies"]);
        let optional_dependencies = extract_dep_map(&snap_v["optionalDependencies"]);

        result.push(PackageInfo {
            name: pkg_name.to_string(),
            version: pkg_version.to_string(),
            target_name,
            tarball_url,
            integrity,
            dependencies,
            optional_dependencies,
            is_dev: false,
        });
    }

    Ok(result)
}

fn extract_from_packages(yaml: &Value, npmrc: &NpmrcConfig) -> Result<Vec<PackageInfo>> {
    let packages_map = yaml["packages"]
        .as_mapping()
        .context("No 'packages' section in lockfile")?;

    let mut result = Vec::new();

    for (k, v) in packages_map {
        let raw_key = k.as_str().unwrap_or_default();
        if raw_key.is_empty() {
            continue;
        }

        let base_key = remove_peer_suffix(raw_key);
        let (pkg_name, pkg_version) = parse_dep_path(base_key);
        let target_name = sanitize_target_name(base_key);

        let tarball_url = if let Some(tarball) = v["resolution"]["tarball"].as_str() {
            Some(tarball.to_string())
        } else if !pkg_name.is_empty() {
            Some(npmrc.tarball_url(pkg_name, pkg_version))
        } else {
            None
        };

        let integrity = v["resolution"]["integrity"].as_str().map(|s| s.to_string());
        let dependencies = extract_dep_map(&v["dependencies"]);
        let optional_dependencies = extract_dep_map(&v["optionalDependencies"]);

        result.push(PackageInfo {
            name: pkg_name.to_string(),
            version: pkg_version.to_string(),
            target_name,
            tarball_url,
            integrity,
            dependencies,
            optional_dependencies,
            is_dev: false,
        });
    }

    Ok(result)
}

fn extract_dep_map(value: &Value) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let Some(mapping) = value.as_mapping() {
        for (name, ver) in mapping {
            let n = name.as_str().unwrap_or_default().to_string();
            let v = ver.as_str().unwrap_or_default().to_string();
            let simple_v = v.split('(').next().unwrap_or(&v).to_string();
            result.push((n, simple_v));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dep_path_scoped() {
        let (name, ver) = parse_dep_path("@sveltejs/kit@2.5.0");
        assert_eq!(name, "@sveltejs/kit");
        assert_eq!(ver, "2.5.0");
    }

    #[test]
    fn test_parse_dep_path_simple() {
        let (name, ver) = parse_dep_path("rollup@4.59.0");
        assert_eq!(name, "rollup");
        assert_eq!(ver, "4.59.0");
    }

    #[test]
    fn test_parse_dep_path_v6_leading_slash() {
        let (name, ver) = parse_dep_path("/@sveltejs/kit@2.5.0");
        assert_eq!(name, "@sveltejs/kit");
        assert_eq!(ver, "2.5.0");
    }

    #[test]
    fn test_remove_peer_suffix() {
        assert_eq!(
            remove_peer_suffix("@sveltejs/kit@2.5.0(svelte@5.0)(vite@7.0)"),
            "@sveltejs/kit@2.5.0"
        );
    }

    #[test]
    fn test_remove_peer_suffix_none() {
        assert_eq!(remove_peer_suffix("rollup@4.59.0"), "rollup@4.59.0");
    }

    #[test]
    fn test_extract_dep_map_strips_peers() {
        let yaml_str = r#"
dependencies:
  vite: 7.3.1(@types/node@22.19.13)
  svelte: 5.53.6
"#;
        let yaml: Value = yaml_serde::from_str(yaml_str).unwrap();
        let deps = extract_dep_map(&yaml["dependencies"]);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0], ("vite".to_string(), "7.3.1".to_string()));
        assert_eq!(deps[1], ("svelte".to_string(), "5.53.6".to_string()));
    }
}
