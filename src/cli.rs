use clap::Parser;

use crate::{procfs, render, tree, user};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "rezky_nightky";
pub const REPOSITORY: &str = "github.com/oxyzenQ/pidnest";

#[derive(Debug, Parser)]
#[command(
    name = "pidnest",
    about = "Minimal Linux process tree viewer by user or UID",
    disable_version_flag = true
)]
pub struct Args {
    /// Username or numeric UID to inspect
    pub user_or_uid: Option<String>,

    /// Show processes owned by the current user
    #[arg(long)]
    pub me: bool,

    /// Print version information
    #[arg(short = 'V', long = "version")]
    pub version: bool,
}

pub fn run(args: Args) -> Result<(), String> {
    if args.version {
        println!("{}", version_text());
        return Ok(());
    }

    let target = match (args.me, args.user_or_uid.as_deref()) {
        (true, None) => user::current_user()?,
        (false, Some(value)) => user::resolve_user_or_uid(value)?,
        (true, Some(_)) => return Err("use either --me or USER_OR_UID, not both".to_string()),
        (false, None) => return Err("expected USER_OR_UID or --me".to_string()),
    };

    let processes = procfs::scan_processes_for_uid(target.uid)?;
    let forest = tree::build_forest(processes);

    if forest.processes.is_empty() {
        println!("No readable processes found for {}.", target.label());
        return Ok(());
    }

    print!("{}", render::render_forest(&target, &forest));
    Ok(())
}

pub fn version_text() -> String {
    format!("pidnest v{VERSION}\n© 2026 {AUTHOR}\nMIT · {REPOSITORY}")
}
