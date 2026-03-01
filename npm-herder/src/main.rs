mod buckify;
mod config;
mod graph;
mod init;
mod lockfile;
mod npmrc;
mod parsers;
mod vendor;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "npm-herder",
    about = "Vendor and buckify npm/pnpm deps for Buck2"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    vendor_dir: Option<String>,

    #[arg(long, global = true)]
    production: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
    Vendor {
        #[arg(short, long)]
        lockfile: Option<String>,
    },
    Buckify {
        #[arg(short, long)]
        lockfile: Option<String>,

        #[arg(long)]
        rules_path: Option<String>,
    },
    Update {
        #[arg(short, long)]
        lockfile: Option<String>,
    },
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::HerderConfig::load(".");
    let npmrc = npmrc::NpmrcConfig::load(".");

    let vendor_dir = cli.vendor_dir.unwrap_or(cfg.vendor_dir);
    let production = cli.production.unwrap_or(cfg.production);

    match &cli.command {
        Commands::Init => {
            init::init(".")?;
        }
        Commands::Vendor { lockfile } => {
            let lockfile = lockfile.as_deref().unwrap_or(&cfg.lockfile);
            let parser = parsers::detect_parser(lockfile);
            let lf = parser.parse(lockfile, &npmrc)?;
            let packages = filter_packages(lf.packages, production);
            println!(
                "Parsed {} lockfile v{} with {} packages",
                lf.manager,
                lf.version,
                packages.len()
            );

            if cfg.vendor.clean_stale {
                vendor::clean_stale_vendors(&packages, &vendor_dir)?;
            }
            vendor::vendor_packages(&packages, &vendor_dir, &npmrc, cfg.vendor.parallel).await?;
        }
        Commands::Buckify {
            lockfile,
            rules_path,
        } => {
            let lockfile = lockfile.as_deref().unwrap_or(&cfg.lockfile);
            let rules_path = rules_path.as_deref().unwrap_or(&cfg.buck.rules_path);
            let parser = parsers::detect_parser(lockfile);
            let lf = parser.parse(lockfile, &npmrc)?;
            let packages = filter_packages(lf.packages, production);
            println!(
                "Parsed {} lockfile v{} with {} packages",
                lf.manager,
                lf.version,
                packages.len()
            );

            let mut dep_graph = graph::DepGraph::build(&packages);
            let broken = dep_graph.detect_and_break_cycles();
            if !broken.is_empty() {
                eprintln!("Broke {} dependency cycle(s):", broken.len());
                for edge in &broken {
                    eprintln!("  {} -/-> {}", edge.from, edge.to);
                }
            }

            buckify::generate_buck_file(&packages, &dep_graph, &vendor_dir, rules_path, &cfg.buck)?;
            println!("Generated {}/BUCK", vendor_dir);
        }
        Commands::Update { lockfile } => {
            let lockfile = lockfile.as_deref().unwrap_or(&cfg.lockfile);
            let rules_path = &cfg.buck.rules_path;
            let parser = parsers::detect_parser(lockfile);
            let lf = parser.parse(lockfile, &npmrc)?;
            let packages = filter_packages(lf.packages, production);
            println!(
                "Parsed {} lockfile v{} with {} packages",
                lf.manager,
                lf.version,
                packages.len()
            );

            if cfg.vendor.clean_stale {
                vendor::clean_stale_vendors(&packages, &vendor_dir)?;
            }
            vendor::vendor_packages(&packages, &vendor_dir, &npmrc, cfg.vendor.parallel).await?;

            let mut dep_graph = graph::DepGraph::build(&packages);
            let broken = dep_graph.detect_and_break_cycles();
            if !broken.is_empty() {
                eprintln!("Broke {} dependency cycle(s):", broken.len());
                for edge in &broken {
                    eprintln!("  {} -/-> {}", edge.from, edge.to);
                }
            }

            buckify::generate_buck_file(&packages, &dep_graph, &vendor_dir, rules_path, &cfg.buck)?;
            println!("Generated {}/BUCK", vendor_dir);
        }
    }

    Ok(())
}

fn filter_packages(
    packages: Vec<lockfile::PackageInfo>,
    production: bool,
) -> Vec<lockfile::PackageInfo> {
    if production {
        packages.into_iter().filter(|p| !p.is_dev).collect()
    } else {
        packages
    }
}
