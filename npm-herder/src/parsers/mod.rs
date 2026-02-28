pub mod pnpm;

use crate::lockfile::LockfileParser;

pub fn detect_parser(lockfile_path: &str) -> Box<dyn LockfileParser> {
    if lockfile_path.contains("pnpm-lock") {
        Box::new(pnpm::PnpmParser)
    } else if lockfile_path.ends_with("package-lock.json") {
        eprintln!("Warning: npm lockfile support is not yet implemented, trying pnpm parser");
        Box::new(pnpm::PnpmParser)
    } else if lockfile_path.ends_with("yarn.lock") {
        eprintln!("Warning: yarn lockfile support is not yet implemented, trying pnpm parser");
        Box::new(pnpm::PnpmParser)
    } else if lockfile_path.contains("bun.lock") {
        eprintln!("Warning: bun lockfile support is not yet implemented, trying pnpm parser");
        Box::new(pnpm::PnpmParser)
    } else if lockfile_path.ends_with("deno.lock") {
        eprintln!("Warning: deno lockfile support is not yet implemented, trying pnpm parser");
        Box::new(pnpm::PnpmParser)
    } else {
        eprintln!("Warning: could not detect package manager, defaulting to pnpm");
        Box::new(pnpm::PnpmParser)
    }
}
