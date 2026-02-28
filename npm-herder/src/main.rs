mod buckify;
mod graph;
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

    #[arg(long, default_value = "vendor", global = true)]
    vendor_dir: String,
}

#[derive(Subcommand)]
enum Commands {
    Vendor {
        #[arg(short, long, default_value = "pnpm-lock.yaml")]
        lockfile: String,
    },
    Buckify {
        #[arg(short, long, default_value = "pnpm-lock.yaml")]
        lockfile: String,

        #[arg(long, default_value = "//rules/js:js_library.bzl")]
        rules_path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let npmrc = npmrc::NpmrcConfig::load(".");

    match &cli.command {
        Commands::Vendor { lockfile } => {
            let parser = parsers::detect_parser(lockfile);
            let lf = parser.parse(lockfile, &npmrc)?;
            println!(
                "Parsed {} lockfile v{} with {} packages",
                lf.manager,
                lf.version,
                lf.packages.len()
            );
            vendor::vendor_packages(&lf.packages, &cli.vendor_dir, &npmrc)?;
        }
        Commands::Buckify {
            lockfile,
            rules_path,
        } => {
            let parser = parsers::detect_parser(lockfile);
            let lf = parser.parse(lockfile, &npmrc)?;
            println!(
                "Parsed {} lockfile v{} with {} packages",
                lf.manager,
                lf.version,
                lf.packages.len()
            );

            let mut dep_graph = graph::DepGraph::build(&lf.packages);

            let broken = dep_graph.detect_and_break_cycles();
            if !broken.is_empty() {
                eprintln!("Broke {} dependency cycle(s):", broken.len());
                for edge in &broken {
                    eprintln!("  {} -/-> {}", edge.from, edge.to);
                }
            }

            buckify::generate_buck_file(&lf.packages, &dep_graph, &cli.vendor_dir, rules_path)?;
            println!("Generated {}/BUCK", cli.vendor_dir);
        }
    }

    Ok(())
}
