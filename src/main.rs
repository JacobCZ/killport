//! The `killport` command-line utility is designed to kill processes
//! listening on specified ports.
//!
//! The utility accepts a list of port numbers as input and attempts to
//! terminate any processes listening on those ports.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
use linux::kill_processes_by_port;
#[cfg(target_os = "macos")]
use macos::kill_processes_by_port;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::{error, Level};
use std::process::exit;

/// The `KillPortArgs` struct is used to parse command-line arguments for the
/// `killport` utility.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct KillPortArgs {
    /// A list of port numbers to kill processes on.
    #[arg(
        name = "ports",
        help = "The list of port numbers to kill processes on",
        required = true
    )]
    ports: Vec<u16>,

    /// A verbosity flag to control the level of logging output.
    #[command(flatten)]
    verbose: Verbosity<WarnLevel>,
    
    /// Show names and PIDs of processes that would be killed but don't actually kill them.
    #[arg(short = 'd', long = "dry-run", default_value_t = false)]
    dry_run: bool
}

/// Indicates the result of the kill operation
pub enum KillResult {
    Killed,
    NotKilled,
    DryRun
}

/// The `main` function is the entry point of the `killport` utility.
///
/// It parses command-line arguments, sets up the logging environment, and
/// attempts to kill processes listening on the specified ports.
fn main() {
    // Parse command-line arguments
    let args = KillPortArgs::parse();

    // Set up logging environment
    let mut log_level = args
        .verbose
        .log_level()
        .map(|level| level.to_level_filter())
        .unwrap();
    
    // If dry-run is enabled, set log level to INFO so we can print out
    // the pids
    if args.dry_run {
        log_level = Level::Info.to_level_filter()
    }

    env_logger::Builder::new()
        .format_module_path(log_level == log::LevelFilter::Trace)
        .format_target(log_level == log::LevelFilter::Trace)
        .format_timestamp(Option::None)
        .filter_level(log_level)
        .init();

    // Attempt to kill processes listening on specified ports
    for port in args.ports {
        match kill_processes_by_port(port, args.dry_run) {
            Ok(killed) => {
                match killed {
                    KillResult::Killed => {
                        println!("Successfully killed process listening on port {}", port);
                    },
                    KillResult::NotKilled => {
                        println!("No processes found using port {}", port);
                    },
                    KillResult::DryRun => {
                        println!("This is a dry-run, no processes were killed.")
                    }
                }
            }
            Err(err) => {
                error!("{}", err);
                exit(1);
            }
        }
    }
}
