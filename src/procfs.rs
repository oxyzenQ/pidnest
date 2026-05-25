use std::fs;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
}

pub fn scan_processes_for_uid(target_uid: u32) -> Result<Vec<Process>, String> {
    let mut processes = Vec::new();
    let entries =
        fs::read_dir("/proc").map_err(|error| format!("failed to read /proc: {error}"))?;

    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse().ok())
        else {
            continue;
        };

        let status_path = entry.path().join("status");
        let Ok(contents) = fs::read_to_string(status_path) else {
            continue;
        };

        let Ok(process) = parse_status(&contents) else {
            continue;
        };

        if process.pid == pid && process.uid == target_uid {
            processes.push(process);
        }
    }

    processes.sort_by_key(|process| process.pid);
    Ok(processes)
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
}
