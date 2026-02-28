use anyhow::Result;
use std::path::Path;

use crate::config;

pub fn detect_package_manager(dir: &str) -> &'static str {
    let dir = Path::new(dir);
    if dir.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if dir.join("package-lock.json").exists() {
        "npm"
    } else if dir.join("yarn.lock").exists() {
        "yarn"
    } else if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() {
        "bun"
    } else if dir.join("deno.lock").exists() {
        "deno"
    } else {
        "unknown"
    }
}

pub fn init(dir: &str) -> Result<()> {
    let config_path = Path::new(dir).join("herder.toml");
    if config_path.exists() {
        println!("herder.toml already exists, skipping");
        return Ok(());
    }

    let pm = detect_package_manager(dir);
    println!("Detected package manager: {}", pm);

    std::fs::write(&config_path, config::DEFAULT_CONFIG)?;
    println!("Created herder.toml");

    println!("\nNext steps:");
    println!("  1. Edit herder.toml to customize settings");
    println!("  2. Run `npm-herder vendor` to download dependencies");
    println!("  3. Run `npm-herder buckify` to generate BUCK targets");
    println!("  Or run `npm-herder update` to do both at once");

    Ok(())
}
