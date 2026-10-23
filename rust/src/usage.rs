//! Resident memory of the app and its WebKit helper processes, read from /proc.

use serde::Serialize;
use std::{collections::HashMap, fs};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Shell process (this binary) in KiB.
    pub app_kib: u64,
    /// All descendant processes (WebKitWebProcess, WebKitNetworkProcess, ...).
    pub web_kib: u64,
    pub processes: usize,
}

fn rss_kib(pid: u32) -> u64 {
    fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("VmRSS:"))
                .and_then(|v| v.split_whitespace().next()?.parse().ok())
        })
        .unwrap_or(0)
}

fn parent_of(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The command name may contain spaces or parens; fields resume after the last ')'.
    stat[stat.rfind(')')? + 2..].split(' ').nth(1)?.parse().ok()
}

pub fn sample() -> Usage {
    let me = std::process::id();
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for pid in entries.flatten().filter_map(|e| e.file_name().to_str()?.parse::<u32>().ok()) {
            if let Some(ppid) = parent_of(pid) {
                children.entry(ppid).or_default().push(pid);
            }
        }
    }

    let mut usage = Usage { app_kib: rss_kib(me), ..Usage::default() };
    let mut stack = children.get(&me).cloned().unwrap_or_default();
    while let Some(pid) = stack.pop() {
        usage.web_kib += rss_kib(pid);
        usage.processes += 1;
        stack.extend(children.get(&pid).into_iter().flatten());
    }
    usage
}
