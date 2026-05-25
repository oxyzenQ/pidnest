use std::collections::{BTreeMap, BTreeSet};

use crate::procfs::Process;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProcessForest {
    pub processes: BTreeMap<u32, Process>,
    pub roots: Vec<u32>,
    pub children: BTreeMap<u32, Vec<u32>>,
}

pub fn build_forest(processes: Vec<Process>) -> ProcessForest {
    let processes: BTreeMap<u32, Process> = processes
        .into_iter()
        .map(|process| (process.pid, process))
        .collect();
    let process_ids: BTreeSet<u32> = processes.keys().copied().collect();
    let mut roots = Vec::new();
    let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();

    for process in processes.values() {
        if process.ppid != process.pid && process_ids.contains(&process.ppid) {
            children.entry(process.ppid).or_default().push(process.pid);
        } else {
            roots.push(process.pid);
        }
    }

    for child_ids in children.values_mut() {
        child_ids.sort_unstable();
    }
    roots.sort_unstable();

    ProcessForest {
        processes,
        roots,
        children,
    }
}

pub fn matching_process_ids(forest: &ProcessForest, pattern: &str) -> BTreeSet<u32> {
    let pattern = pattern.to_lowercase();
    forest
        .processes
        .values()
        .filter(|process| process.name.to_lowercase().contains(&pattern))
        .map(|process| process.pid)
        .collect()
}

pub fn prune_for_matches(forest: &ProcessForest, matches: &BTreeSet<u32>) -> ProcessForest {
    let mut included = BTreeSet::new();

    for pid in matches {
        include_ancestors(forest, *pid, &mut included);
        include_descendants(forest, *pid, &mut included);
    }

    forest_from_ids(forest, &included)
}

pub fn limit_depth(forest: &ProcessForest, max_depth: Option<usize>) -> ProcessForest {
    let Some(max_depth) = max_depth else {
        return forest.clone();
    };

    if max_depth == 0 {
        return build_forest(Vec::new());
    }

    let mut included = BTreeSet::new();
    for root in &forest.roots {
        include_to_depth(forest, *root, 1, max_depth, &mut included);
    }

    forest_from_ids(forest, &included)
}

fn include_ancestors(forest: &ProcessForest, pid: u32, included: &mut BTreeSet<u32>) {
    let mut current_pid = pid;
    while let Some(process) = forest.processes.get(&current_pid) {
        included.insert(process.pid);
        if process.ppid == process.pid || !forest.processes.contains_key(&process.ppid) {
            break;
        }
        current_pid = process.ppid;
    }
}

fn include_descendants(forest: &ProcessForest, pid: u32, included: &mut BTreeSet<u32>) {
    included.insert(pid);

    if let Some(children) = forest.children.get(&pid) {
        for child in children {
            include_descendants(forest, *child, included);
        }
    }
}

fn include_to_depth(
    forest: &ProcessForest,
    pid: u32,
    depth: usize,
    max_depth: usize,
    included: &mut BTreeSet<u32>,
) {
    if depth > max_depth || !included.insert(pid) {
        return;
    }

    if let Some(children) = forest.children.get(&pid) {
        for child in children {
            include_to_depth(forest, *child, depth + 1, max_depth, included);
        }
    }
}

fn forest_from_ids(forest: &ProcessForest, included: &BTreeSet<u32>) -> ProcessForest {
    build_forest(
        included
            .iter()
            .filter_map(|pid| forest.processes.get(pid).cloned())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(name: &str, pid: u32, ppid: u32) -> Process {
        Process {
            name: name.to_string(),
            pid,
            ppid,
            uid: 1000,
        }
    }

    #[test]
    fn treats_process_as_root_when_parent_is_not_in_filtered_set() {
        let forest = build_forest(vec![
            process("bash", 20, 1),
            process("python3", 30, 20),
            process("cargo", 25, 20),
        ]);

        assert_eq!(forest.roots, vec![20]);
        assert_eq!(forest.children.get(&20), Some(&vec![25, 30]));
    }

    #[test]
    fn treats_self_parent_as_root() {
        let forest = build_forest(vec![process("initlike", 1, 1)]);

        assert_eq!(forest.roots, vec![1]);
        assert!(forest.children.is_empty());
    }

    #[test]
    fn finds_process_names_case_insensitively() {
        let forest = build_forest(vec![
            process("bash", 20, 1),
            process("Codex", 30, 20),
            process("cargo", 40, 20),
        ]);

        assert_eq!(matching_process_ids(&forest, "codex"), BTreeSet::from([30]));
    }

    #[test]
    fn pruning_keeps_ancestors_of_matches() {
        let forest = build_forest(vec![
            process("bash", 20, 1),
            process("python3", 30, 20),
            process("codex", 40, 30),
        ]);
        let pruned = prune_for_matches(&forest, &BTreeSet::from([40]));

        assert_eq!(pruned.roots, vec![20]);
        assert!(pruned.processes.contains_key(&20));
        assert!(pruned.processes.contains_key(&30));
        assert!(pruned.processes.contains_key(&40));
    }

    #[test]
    fn pruning_keeps_descendants_of_matches() {
        let forest = build_forest(vec![
            process("bash", 20, 1),
            process("codex", 30, 20),
            process("worker", 40, 30),
        ]);
        let pruned = prune_for_matches(&forest, &BTreeSet::from([30]));

        assert_eq!(pruned.roots, vec![20]);
        assert!(pruned.processes.contains_key(&20));
        assert!(pruned.processes.contains_key(&30));
        assert!(pruned.processes.contains_key(&40));
    }

    #[test]
    fn limits_depth_zero_to_no_processes() {
        let forest = build_forest(vec![process("bash", 20, 1)]);
        let limited = limit_depth(&forest, Some(0));

        assert!(limited.roots.is_empty());
        assert!(limited.processes.is_empty());
    }

    #[test]
    fn limits_depth_one_to_roots_only() {
        let forest = build_forest(vec![process("bash", 20, 1), process("python3", 30, 20)]);
        let limited = limit_depth(&forest, Some(1));

        assert_eq!(limited.roots, vec![20]);
        assert_eq!(limited.processes.len(), 1);
        assert!(limited.processes.contains_key(&20));
    }

    #[test]
    fn limits_depth_two_to_roots_and_direct_children() {
        let forest = build_forest(vec![
            process("bash", 20, 1),
            process("python3", 30, 20),
            process("worker", 40, 30),
        ]);
        let limited = limit_depth(&forest, Some(2));

        assert_eq!(limited.roots, vec![20]);
        assert_eq!(limited.processes.len(), 2);
        assert!(limited.processes.contains_key(&20));
        assert!(limited.processes.contains_key(&30));
        assert!(!limited.processes.contains_key(&40));
    }
}
