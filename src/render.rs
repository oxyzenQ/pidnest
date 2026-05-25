use owo_colors::{OwoColorize, Style};

use crate::{tree::ProcessForest, user::TargetUser};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenderOptions {
    pub show_pid: bool,
    pub color: bool,
}

pub fn render_forest(
    target: &TargetUser,
    forest: &ProcessForest,
    options: RenderOptions,
) -> String {
    let mut output = format!(
        "{}\n",
        paint(&target.label(), Style::new().bold(), options.color)
    );

    for (index, pid) in forest.roots.iter().enumerate() {
        render_process(
            &mut output,
            forest,
            *pid,
            "",
            index + 1 == forest.roots.len(),
            options,
        );
    }

    output
}

pub fn render_summary(forest: &ProcessForest, options: RenderOptions) -> String {
    paint(
        &format!(
            "{} roots · {} processes",
            forest.roots.len(),
            forest.processes.len()
        ),
        Style::new().dimmed(),
        options.color,
    )
}

pub fn render_unreadable_note(options: RenderOptions) -> String {
    paint(
        "note: some processes were not readable; try sudo for a complete tree",
        Style::new().dimmed(),
        options.color,
    )
}

pub fn render_live_footer(interval_seconds: u64, options: RenderOptions) -> String {
    paint(
        &format!("live mode · refresh {interval_seconds}s · press Ctrl+C to quit"),
        Style::new().dimmed(),
        options.color,
    )
}

fn render_process(
    output: &mut String,
    forest: &ProcessForest,
    pid: u32,
    prefix: &str,
    is_last: bool,
    options: RenderOptions,
) {
    let Some(process) = forest.processes.get(&pid) else {
        return;
    };

    let branch = if is_last { "└── " } else { "├── " };
    output.push_str(&paint(
        &format!("{prefix}{branch}"),
        Style::new().dimmed(),
        options.color,
    ));
    output.push_str(&process.name);

    if options.show_pid {
        output.push_str(&paint(
            &format!(" pid={}", process.pid),
            Style::new().dimmed(),
            options.color,
        ));
    }

    output.push('\n');

    let next_prefix = if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}│   ")
    };

    let Some(children) = forest.children.get(&pid) else {
        return;
    };

    for (index, child_pid) in children.iter().enumerate() {
        render_process(
            output,
            forest,
            *child_pid,
            &next_prefix,
            index + 1 == children.len(),
            options,
        );
    }
}

fn paint(text: &str, style: Style, enabled: bool) -> String {
    if enabled {
        text.style(style).to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{procfs::Process, tree::build_forest};

    use super::*;

    fn target() -> TargetUser {
        TargetUser {
            name: "rezky".to_string(),
            uid: 1000,
        }
    }

    fn forest() -> ProcessForest {
        build_forest(vec![
            Process {
                name: "bash".to_string(),
                pid: 20,
                ppid: 1,
                uid: 1000,
            },
            Process {
                name: "python3".to_string(),
                pid: 30,
                ppid: 20,
                uid: 1000,
            },
            Process {
                name: "cargo".to_string(),
                pid: 25,
                ppid: 20,
                uid: 1000,
            },
        ])
    }

    #[test]
    fn renders_tree_with_pid() {
        let output = render_forest(
            &target(),
            &forest(),
            RenderOptions {
                show_pid: true,
                color: false,
            },
        );

        assert_eq!(
            output,
            "rezky uid=1000\n└── bash pid=20\n    ├── cargo pid=25\n    └── python3 pid=30\n"
        );
    }

    #[test]
    fn renders_tree_without_pid() {
        let output = render_forest(
            &target(),
            &forest(),
            RenderOptions {
                show_pid: false,
                color: false,
            },
        );

        assert_eq!(
            output,
            "rezky uid=1000\n└── bash\n    ├── cargo\n    └── python3\n"
        );
    }

    #[test]
    fn disabled_color_has_no_ansi_escape_codes() {
        let output = render_forest(
            &target(),
            &forest(),
            RenderOptions {
                show_pid: true,
                color: false,
            },
        );

        assert!(!output.contains("\u{1b}["));
    }

    #[test]
    fn renders_stable_summary() {
        assert_eq!(
            render_summary(
                &forest(),
                RenderOptions {
                    show_pid: true,
                    color: false,
                }
            ),
            "1 roots · 3 processes"
        );
    }
}
