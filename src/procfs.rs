use std::{fs, path::Path};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScanReport {
    pub processes: Vec<Process>,
    pub unreadable_statuses: usize,
}

pub fn scan_processes_for_uid(target_uid: u32) -> Result<ScanReport, String> {
    scan_processes_for_uid_at(Path::new("/proc"), target_uid)
}

fn scan_processes_for_uid_at(proc_root: &Path, target_uid: u32) -> Result<ScanReport, String> {
    let mut processes = Vec::new();
    let mut unreadable_statuses = 0;
    let entries = fs::read_dir(proc_root)
        .map_err(|error| format!("failed to read {}: {error}", proc_root.display()))?;

    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse().ok())
        else {
            continue;
        };

        let status_path = entry.path().join("status");
        let contents = match fs::read_to_string(status_path) {
            Ok(contents) => contents,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    unreadable_statuses += 1;
                }
                continue;
            }
        };

        let Ok(process) = parse_status(&contents) else {
            continue;
        };

        if process.pid == pid && process.uid == target_uid {
            processes.push(process);
        }
    }

    processes.sort_by_key(|process| process.pid);
    Ok(ScanReport {
        processes,
        unreadable_statuses,
    })
}

pub fn parse_status(contents: &str) -> Result<Process, String> {
    let mut name = None;
    let mut pid = None;
    let mut ppid = None;
    let mut uid = None;

    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("Name:") {
            name = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("Pid:") {
            pid = parse_number(value, "Pid")?;
        } else if let Some(value) = line.strip_prefix("PPid:") {
            ppid = parse_number(value, "PPid")?;
        } else if let Some(value) = line.strip_prefix("Uid:") {
            uid = value
                .split_whitespace()
                .next()
                .ok_or_else(|| "missing real UID".to_string())?
                .parse::<u32>()
                .map(Some)
                .map_err(|_| "invalid real UID".to_string())?;
        }
    }

    Ok(Process {
        name: name.ok_or_else(|| "missing Name".to_string())?,
        pid: pid.ok_or_else(|| "missing Pid".to_string())?,
        ppid: ppid.ok_or_else(|| "missing PPid".to_string())?,
        uid: uid.ok_or_else(|| "missing Uid".to_string())?,
    })
}

fn parse_number(value: &str, field: &str) -> Result<Option<u32>, String> {
    value
        .trim()
        .parse::<u32>()
        .map(Some)
        .map_err(|_| format!("invalid {field}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_linux_status_fields() {
        let contents = "\
Name:\tbash
Umask:\t0022
State:\tS (sleeping)
Pid:\t1234
PPid:\t1000
Uid:\t1000\t1000\t1000\t1000
";

        let process = parse_status(contents).expect("status should parse");

        assert_eq!(
            process,
            Process {
                name: "bash".to_string(),
                pid: 1234,
                ppid: 1000,
                uid: 1000,
            }
        );
    }

    #[test]
    fn rejects_status_without_required_fields() {
        let error = parse_status("Name:\tbash\nPid:\t1234\n").expect_err("status should fail");

        assert_eq!(error, "missing PPid");
    }

    #[test]
    fn tracks_unreadable_statuses_without_real_proc() {
        let proc_root = std::env::temp_dir().join(format!("pidnest-test-{}", std::process::id()));
        let readable_pid = proc_root.join("1234");
        let unreadable_pid = proc_root.join("5678");
        fs::create_dir_all(&readable_pid).expect("readable pid dir");
        fs::create_dir_all(unreadable_pid.join("status")).expect("unreadable status path");
        fs::write(
            readable_pid.join("status"),
            "Name:\tbash\nPid:\t1234\nPPid:\t1\nUid:\t1000\t1000\t1000\t1000\n",
        )
        .expect("status file");

        let report = scan_processes_for_uid_at(&proc_root, 1000).expect("scan should succeed");

        assert_eq!(report.processes.len(), 1);
        assert_eq!(report.unreadable_statuses, 1);

        fs::remove_dir_all(proc_root).expect("cleanup");
    }
}
