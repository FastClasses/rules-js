use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct NpmrcConfig {
    pub default_registry: String,
    pub scoped_registries: HashMap<String, String>,
    pub auth_tokens: HashMap<String, String>,
}

impl NpmrcConfig {
    pub fn load(dir: &str) -> Self {
        let npmrc_path = Path::new(dir).join(".npmrc");
        if !npmrc_path.exists() {
            return Self::default_config();
        }

        match std::fs::read_to_string(&npmrc_path) {
            Ok(content) => parse_npmrc(&content),
            Err(_) => Self::default_config(),
        }
    }

    pub fn registry_for(&self, package_name: &str) -> &str {
        if package_name.starts_with('@') {
            if let Some(scope) = package_name.split('/').next() {
                if let Some(registry) = self.scoped_registries.get(scope) {
                    return registry;
                }
            }
        }
        &self.default_registry
    }

    pub fn auth_token_for(&self, registry_url: &str) -> Option<&str> {
        for (host_path, token) in &self.auth_tokens {
            if registry_url.contains(host_path.trim_start_matches('/')) {
                return Some(token);
            }
        }
        None
    }

    pub fn tarball_url(&self, package_name: &str, version: &str) -> String {
        let registry = self.registry_for(package_name);
        let registry = registry.trim_end_matches('/');
        let basename = package_name.split('/').last().unwrap_or(package_name);
        format!(
            "{}/{}/-/{}-{}.tgz",
            registry, package_name, basename, version
        )
    }

    fn default_config() -> Self {
        NpmrcConfig {
            default_registry: "https://registry.npmjs.org".to_string(),
            scoped_registries: HashMap::new(),
            auth_tokens: HashMap::new(),
        }
    }
}

fn parse_npmrc(content: &str) -> NpmrcConfig {
    let mut config = NpmrcConfig::default_config();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('@') && line.contains(":registry=") {
            if let Some((scope_part, url)) = line.split_once(":registry=") {
                let scope = scope_part.trim().to_string();
                let url = url.trim().to_string();
                config.scoped_registries.insert(scope, url);
            }
            continue;
        }

        if line.starts_with("//") && line.contains(":_authToken=") {
            if let Some((host_path, token_raw)) = line.split_once(":_authToken=") {
                let token = expand_env_var(token_raw.trim());
                config.auth_tokens.insert(host_path.to_string(), token);
            }
            continue;
        }

        if line.starts_with("registry=") {
            config.default_registry = line.trim_start_matches("registry=").trim().to_string();
            continue;
        }
    }

    config
}

fn expand_env_var(s: &str) -> String {
    if s.starts_with("${") && s.ends_with('}') {
        let var_name = &s[2..s.len() - 1];
        std::env::var(var_name).unwrap_or_else(|_| {
            eprintln!(
                "Warning: environment variable {} not set (referenced in .npmrc)",
                var_name
            );
            String::new()
        })
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scoped_registry() {
        let content = "@fastclasses:registry=https://npm.pkg.github.com/\n";
        let config = parse_npmrc(content);
        assert_eq!(
            config.scoped_registries.get("@fastclasses"),
            Some(&"https://npm.pkg.github.com/".to_string())
        );
    }

    #[test]
    fn test_parse_default_registry() {
        let content = "registry=https://custom.registry.example.com/\n";
        let config = parse_npmrc(content);
        assert_eq!(
            config.default_registry,
            "https://custom.registry.example.com/"
        );
    }

    #[test]
    fn test_parse_auth_token_literal() {
        let content = "//npm.pkg.github.com/:_authToken=ghp_abc123\n";
        let config = parse_npmrc(content);
        assert_eq!(
            config.auth_tokens.get("//npm.pkg.github.com/"),
            Some(&"ghp_abc123".to_string())
        );
    }

    #[test]
    fn test_registry_for_scoped() {
        let content = "@fastclasses:registry=https://npm.pkg.github.com/\n";
        let config = parse_npmrc(content);
        assert_eq!(
            config.registry_for("@fastclasses/my-lib"),
            "https://npm.pkg.github.com/"
        );
    }

    #[test]
    fn test_registry_for_unscoped() {
        let content = "@fastclasses:registry=https://npm.pkg.github.com/\n";
        let config = parse_npmrc(content);
        assert_eq!(config.registry_for("rollup"), "https://registry.npmjs.org");
    }

    #[test]
    fn test_tarball_url_scoped() {
        let content = "@fastclasses:registry=https://npm.pkg.github.com/\n";
        let config = parse_npmrc(content);
        assert_eq!(
            config.tarball_url("@fastclasses/my-lib", "1.0.0"),
            "https://npm.pkg.github.com/@fastclasses/my-lib/-/my-lib-1.0.0.tgz"
        );
    }

    #[test]
    fn test_tarball_url_default() {
        let config = NpmrcConfig::default_config();
        assert_eq!(
            config.tarball_url("rollup", "4.59.0"),
            "https://registry.npmjs.org/rollup/-/rollup-4.59.0.tgz"
        );
    }

    #[test]
    fn test_auth_token_for_registry() {
        let content = "//npm.pkg.github.com/:_authToken=ghp_abc123\n";
        let config = parse_npmrc(content);
        assert_eq!(
            config.auth_token_for("https://npm.pkg.github.com/@fastclasses/my-lib"),
            Some("ghp_abc123")
        );
    }

    #[test]
    fn test_skips_comments() {
        let content = "# comment\n; another comment\n@scope:registry=https://example.com/\n";
        let config = parse_npmrc(content);
        assert_eq!(config.scoped_registries.len(), 1);
    }
}
