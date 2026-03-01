use anyhow::Result;

use crate::npmrc::NpmrcConfig;

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub target_name: String,
    pub tarball_url: Option<String>,
    pub integrity: Option<String>,
    pub dependencies: Vec<(String, String)>,
    pub optional_dependencies: Vec<(String, String)>,
    pub is_dev: bool,
}

pub struct Lockfile {
    pub version: String,
    pub manager: String,
    pub packages: Vec<PackageInfo>,
}

pub trait LockfileParser {
    fn parse(&self, path: &str, npmrc: &NpmrcConfig) -> Result<Lockfile>;
}

pub fn sanitize_target_name(pkg_path: &str) -> String {
    pkg_path
        .replace('/', "_")
        .replace('@', "_")
        .replace('+', "_")
        .trim_start_matches('_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_target_name_scoped() {
        assert_eq!(
            sanitize_target_name("/@sveltejs/kit@2.5.0"),
            "sveltejs_kit_2.5.0"
        );
    }

    #[test]
    fn test_sanitize_target_name_simple() {
        assert_eq!(sanitize_target_name("/rollup@4.59.0"), "rollup_4.59.0");
    }

    #[test]
    fn test_sanitize_target_name_no_leading_slash() {
        assert_eq!(sanitize_target_name("rollup@4.59.0"), "rollup_4.59.0");
    }

    #[test]
    fn test_sanitize_target_name_plus() {
        assert_eq!(sanitize_target_name("/@foo/bar+baz@1.0"), "foo_bar_baz_1.0");
    }
}
