use anyhow::Result;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::config::BuckConfig;
use crate::graph::DepGraph;
use crate::lockfile::PackageInfo;

#[derive(serde::Serialize)]
#[serde(rename = "js_library")]
struct JsLibrary {
    name: String,
    package_name: String,
    version: String,
    srcs: Glob,
    deps: Vec<String>,
    visibility: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename = "glob")]
struct Glob(Vec<String>);

pub fn generate_buck_file(
    packages: &[PackageInfo],
    graph: &DepGraph,
    vendor_dir: &str,
    rules_path: &str,
    buck_config: &BuckConfig,
) -> Result<()> {
    let mut content = String::new();

    if !buck_config.generated_file_header.is_empty() {
        content.push_str(&buck_config.generated_file_header);
        if !content.ends_with('\n') {
            content.push('\n');
        }
    }

    content.push_str(&format!("load(\"{}\", \"js_library\")\n\n", rules_path,));

    for pkg in packages {
        let deps = graph.get_deps(&pkg.target_name);

        let js_lib = JsLibrary {
            name: pkg.target_name.clone(),
            package_name: pkg.name.clone(),
            version: pkg.version.clone(),
            srcs: Glob(vec![format!("{}/**", pkg.target_name)]),
            deps,
            visibility: vec!["PUBLIC".to_string()],
        };

        content.push_str(&serde_starlark::to_string(&js_lib)?);
        content.push_str("\n\n");
    }

    let vendor_path = PathBuf::from(vendor_dir);
    create_dir_all(&vendor_path)?;
    std::fs::write(vendor_path.join(&buck_config.file_name), content)?;

    Ok(())
}
