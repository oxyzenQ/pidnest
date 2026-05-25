use std::{
    io::{self, IsTerminal, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use clap::Parser;

use crate::{
    procfs::{self, ScanReport},
    render::{self, RenderOptions},
    tree, user,
    user::TargetUser,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "rezky_nightky";
pub const REPOSITORY: &str = "github.com/oxyzenQ/pidnest";
pub const DEFAULT_INTERVAL_SECONDS: u64 = 6;
pub const MIN_INTERVAL_SECONDS: u64 = 3;
pub const MAX_INTERVAL_SECONDS: u64 = 60;

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

    /// Continuously refresh the process tree
    #[arg(long)]
    pub live: bool,

    /// Alias for --live
    #[arg(long)]
    pub watch: bool,

    /// Live refresh interval in seconds
    #[arg(long)]
    pub interval: Option<u64>,

    /// Limit rendered tree depth, where 0 shows only header and summary
    #[arg(long, allow_hyphen_values = true)]
    pub depth: Option<usize>,

    /// Show process families matching a case-insensitive name substring
    #[arg(long)]
    pub find: Option<String>,

    /// Hide pid=<PID> labels
    #[arg(long)]
    pub no_pid: bool,

    /// Disable ANSI color output
    #[arg(long)]
    pub no_color: bool,

    /// Print version information
    #[arg(short = 'V', long = "version")]
    pub version: bool,
}

pub fn run(args: Args) -> Result<(), String> {
    if args.version {
        println!("{}", version_text());
        return Ok(());
    }

    let live = is_live_mode(args.live, args.watch);
    let interval_seconds = validate_interval(live, args.interval)?;
    let find = validate_find(args.find)?;
    let target = match (args.me, args.user_or_uid.as_deref()) {
        (true, None) => user::current_user()?,
        (false, Some(value)) => user::resolve_user_or_uid(value)?,
        (true, Some(_)) => return Err("use either --me or USER_OR_UID, not both".to_string()),
        (false, None) => return Err("expected USER_OR_UID or --me".to_string()),
    };
    let options = RenderOptions {
        show_pid: !args.no_pid,
        color: should_use_color(args.no_color),
    };
    let view = ViewOptions {
        max_depth: args.depth,
        find,
    };

    if live {
        run_live(target, interval_seconds, options, view)
    } else {
        print!("{}", render_snapshot(&target, options, &view, None)?);
        Ok(())
    }
}

pub fn version_text() -> String {
    version_text_with_hash(option_env!("PIDNEST_GIT_HASH").unwrap_or("unknown"))
}

pub fn version_text_with_hash(git_hash: &str) -> String {
    let git_hash = if git_hash.trim().is_empty() {
        "unknown"
    } else {
        git_hash.trim()
    };

    format!("pidnest v{VERSION} ({git_hash})\n© 2026 {AUTHOR}\nMIT · {REPOSITORY}")
}

pub fn validate_interval(live: bool, interval: Option<u64>) -> Result<u64, String> {
    match (live, interval) {
        (false, Some(_)) => Err("--interval requires --live".to_string()),
        (false, None) => Ok(DEFAULT_INTERVAL_SECONDS),
        (true, Some(seconds))
            if !(MIN_INTERVAL_SECONDS..=MAX_INTERVAL_SECONDS).contains(&seconds) =>
        {
            Err(format!(
                "--interval must be between {MIN_INTERVAL_SECONDS} and {MAX_INTERVAL_SECONDS} seconds"
            ))
        }
        (true, Some(seconds)) => Ok(seconds),
        (true, None) => Ok(DEFAULT_INTERVAL_SECONDS),
    }
}

pub fn is_live_mode(live: bool, watch: bool) -> bool {
    live || watch
}

fn validate_find(find: Option<String>) -> Result<Option<String>, String> {
    match find {
        Some(pattern) if pattern.trim().is_empty() => {
            Err("--find requires a non-empty pattern".to_string())
        }
        Some(pattern) => Ok(Some(pattern)),
        None => Ok(None),
    }
}

fn should_use_color(no_color: bool) -> bool {
    !no_color && std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal()
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ViewOptions {
    max_depth: Option<usize>,
    find: Option<String>,
}

fn run_live(
    target: TargetUser,
    interval_seconds: u64,
    options: RenderOptions,
    view: ViewOptions,
) -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    let handler_running = Arc::clone(&running);
    ctrlc::set_handler(move || {
        handler_running.store(false, Ordering::SeqCst);
    })
    .map_err(|error| format!("failed to install Ctrl+C handler: {error}"))?;

    let interval = Duration::from_secs(interval_seconds);
    while running.load(Ordering::SeqCst) {
        print!("\x1b[2J\x1b[H");
        print!(
            "{}",
            render_snapshot(&target, options, &view, Some(interval_seconds))?
        );
        io::stdout()
            .flush()
            .map_err(|error| format!("failed to flush stdout: {error}"))?;

        let started = Instant::now();
        while running.load(Ordering::SeqCst) && started.elapsed() < interval {
            let remaining = interval.saturating_sub(started.elapsed());
            thread::sleep(remaining.min(Duration::from_millis(200)));
        }
    }

    Ok(())
}

fn render_snapshot(
    target: &TargetUser,
    options: RenderOptions,
    view: &ViewOptions,
    live_interval_seconds: Option<u64>,
) -> Result<String, String> {
    let report = procfs::scan_processes_for_uid(target.uid)?;
    Ok(render_report(
        target,
        report,
        options,
        view,
        live_interval_seconds,
    ))
}

fn render_report(
    target: &TargetUser,
    report: ScanReport,
    options: RenderOptions,
    view: &ViewOptions,
    live_interval_seconds: Option<u64>,
) -> String {
    let mut forest = tree::build_forest(report.processes);
    if forest.processes.is_empty() {
        let mut output = if target.uid == 0 {
            "No readable processes found for uid=0. Try: sudo pidnest root\n".to_string()
        } else {
            format!("No readable processes found for {}.\n", target.label())
        };
        if let Some(interval_seconds) = live_interval_seconds {
            output.push('\n');
            output.push_str(&render::render_live_footer(interval_seconds, options));
            output.push('\n');
        }
        return output;
    }

    if let Some(pattern) = &view.find {
        let matches = tree::matching_process_ids(&forest, pattern);
        if matches.is_empty() {
            let mut output = format!("No processes matched '{pattern}'.\n");
            if let Some(interval_seconds) = live_interval_seconds {
                output.push('\n');
                output.push_str(&render::render_live_footer(interval_seconds, options));
                output.push('\n');
            }
            return output;
        }
        forest = tree::prune_for_matches(&forest, &matches);
    }

    forest = tree::limit_depth(&forest, view.max_depth);

    let mut output = render::render_forest(target, &forest, options);
    output.push('\n');

    if report.unreadable_statuses > 0 {
        output.push_str(&render::render_unreadable_note(options));
        output.push('\n');
    }

    output.push_str(&render::render_summary(&forest, options));
    output.push('\n');

    if let Some(interval_seconds) = live_interval_seconds {
        output.push_str(&render::render_live_footer(interval_seconds, options));
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::procfs::Process;

    use super::*;

    fn target() -> TargetUser {
        TargetUser {
            name: "rezky".to_string(),
            uid: 1000,
        }
    }

    fn process(name: &str, pid: u32, ppid: u32) -> Process {
        Process {
            name: name.to_string(),
            pid,
            ppid,
            uid: 1000,
        }
    }

    fn report() -> ScanReport {
        ScanReport {
            processes: vec![
                process("bash", 20, 1),
                process("python3", 30, 20),
                process("codex", 40, 30),
                process("worker", 50, 40),
            ],
            unreadable_statuses: 0,
        }
    }

    fn options(show_pid: bool) -> RenderOptions {
        RenderOptions {
            show_pid,
            color: false,
        }
    }

    fn view(max_depth: Option<usize>, find: Option<&str>) -> ViewOptions {
        ViewOptions {
            max_depth,
            find: find.map(str::to_string),
        }
    }

    #[test]
    fn accepts_default_interval_for_live_mode() {
        assert_eq!(validate_interval(true, None), Ok(DEFAULT_INTERVAL_SECONDS));
    }

    #[test]
    fn accepts_interval_bounds_for_live_mode() {
        assert_eq!(validate_interval(true, Some(MIN_INTERVAL_SECONDS)), Ok(3));
        assert_eq!(validate_interval(true, Some(MAX_INTERVAL_SECONDS)), Ok(60));
    }

    #[test]
    fn rejects_interval_outside_bounds() {
        assert_eq!(
            validate_interval(true, Some(2)),
            Err("--interval must be between 3 and 60 seconds".to_string())
        );
        assert_eq!(
            validate_interval(true, Some(61)),
            Err("--interval must be between 3 and 60 seconds".to_string())
        );
    }

    #[test]
    fn rejects_interval_without_live_mode() {
        assert_eq!(
            validate_interval(false, Some(6)),
            Err("--interval requires --live".to_string())
        );
    }

    #[test]
    fn watch_behaves_like_live_for_interval_validation() {
        assert!(is_live_mode(false, true));
        assert_eq!(validate_interval(is_live_mode(false, true), Some(3)), Ok(3));
    }

    #[test]
    fn depth_zero_renders_header_and_summary_only() {
        let output = render_report(
            &target(),
            report(),
            options(true),
            &view(Some(0), None),
            None,
        );

        assert_eq!(output, "rezky uid=1000\n\n0 roots · 0 processes\n");
    }

    #[test]
    fn depth_combines_with_no_pid() {
        let output = render_report(
            &target(),
            report(),
            options(false),
            &view(Some(2), None),
            None,
        );

        assert_eq!(
            output,
            "rezky uid=1000\n└── bash\n    └── python3\n\n1 roots · 2 processes\n"
        );
        assert!(!output.contains(" pid="));
        assert!(!output.contains("\u{1b}["));
    }

    #[test]
    fn find_no_match_returns_clean_message() {
        let output = render_report(
            &target(),
            report(),
            options(true),
            &view(None, Some("missing")),
            None,
        );

        assert_eq!(output, "No processes matched 'missing'.\n");
    }

    #[test]
    fn formats_version_metadata_with_hash() {
        let output = version_text_with_hash("abc1234");
        let expected_version = format!("v{}", env!("CARGO_PKG_VERSION"));

        assert!(output.contains("pidnest"));
        assert!(output.contains(&expected_version));
        assert!(output.contains("(abc1234)"));
        assert!(output.contains("rezky_nightky"));
        assert!(output.contains("MIT"));
        assert!(output.contains("github.com/oxyzenQ/pidnest"));
    }

    #[test]
    fn formats_version_metadata_with_unknown_fallback() {
        let output = version_text_with_hash("");
        let expected_version = format!("v{}", env!("CARGO_PKG_VERSION"));

        assert!(output.contains("pidnest"));
        assert!(output.contains(&expected_version));
        assert!(output.contains("(unknown)"));
        assert!(output.contains("rezky_nightky"));
        assert!(output.contains("MIT"));
        assert!(output.contains("github.com/oxyzenQ/pidnest"));
    }
}
