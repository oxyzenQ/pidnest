use crate::{tree::ProcessForest, user::TargetUser};

pub fn render_forest(target: &TargetUser, forest: &ProcessForest) -> String {
    let mut output = format!("{}\n", target.label());

    for (index, pid) in forest.roots.iter().enumerate() {
        render_process(
            &mut output,
            forest,
            *pid,
            "",
            index + 1 == forest.roots.len(),
        );
    }

    output
}

fn render_process(
    output: &mut String,
    forest: &ProcessForest,
    pid: u32,
    prefix: &str,
    is_last: bool,
) {
    let Some(process) = forest.processes.get(&pid) else {
        return;
    };

    let branch = if is_last { "└── " } else { "├── " };
    output.push_str(prefix);
    output.push_str(branch);
    output.push_str(&process.name);
    output.push_str(" pid=");
    output.push_str(&process.pid.to_string());
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
        );
    }
}
