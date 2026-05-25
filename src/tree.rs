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
}
