#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
package.edition = "2024"

[dependencies]
---

//! Commit the oldest uncommitted daily worklog block with jj.
//!
//! Usage:
//!   commit-daily-logs.rs [--commit]
//!
//! Behavior:
//! - Finds the oldest uncommitted date block after the latest "log:" commit.
//! - Reconstructs worklogs/daily.md as @- plus the target block, commits it,
//!   then restores the full content into the working copy.
//! - Changes outside worklogs/daily.md stay in the working copy.
//! - The working copy is snapshotted up front, so any mid-run failure is
//!   recoverable via `jj op restore`.

use std::env;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::fs;
use std::process;
use std::process::Command;
use std::process::Stdio;

type BoxResult<T> = Result<T, Box<dyn error::Error>>;

const DAILY_PATH: &str = "worklogs/daily.md";

#[derive(Debug)]
struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.0)
    }
}

impl error::Error for Error {}

fn error(msg: impl Into<String>) -> Box<Error> {
    Box::new(Error(msg.into()))
}

fn jj(args: &[&str]) -> BoxResult<String> {
    let output = Command::new("jj")
        .arg("--quiet")
        .args(args)
        .stdout(Stdio::piped())
        .output()?;
    if !output.status.success() {
        return Err(error(format!("command failed: jj {}", args.join(" "))));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    bytes
        .iter()
        .enumerate()
        .all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit())
}

fn split_lines(input: &str) -> (Vec<String>, bool) {
    let ends_with_newline = input.ends_with('\n');
    let lines = input.lines().map(|line| line.to_string()).collect();
    (lines, ends_with_newline)
}

fn validate_header(lines: &[String], source: &str) -> BoxResult<()> {
    if lines.len() < 2 {
        return Err(error(format!("{source} is too short")));
    }
    if lines[0] != "# Daily worklogs" || !lines[1].is_empty() {
        return Err(error(format!("{source} has unexpected header")));
    }
    Ok(())
}

#[derive(Debug)]
struct Heading {
    date: String,
    line_idx: usize,
}

fn collect_headings(lines: &[String]) -> BoxResult<Vec<Heading>> {
    let mut headings = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let Some(date) = line.strip_prefix("## ") else {
            continue;
        };
        let date = date.trim();
        if !is_date(date) {
            return Err(error(format!(
                "invalid date format at line {}: {date}",
                idx + 1
            )));
        }
        headings.push(Heading {
            date: date.to_string(),
            line_idx: idx,
        });
    }
    Ok(headings)
}

fn main() -> BoxResult<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut dry_run = true;
    if args.len() > 1 {
        eprintln!("Usage: commit-daily-logs.rs [--commit]");
        process::exit(1);
    }
    if let Some(flag) = args.first() {
        if flag == "--commit" {
            dry_run = false;
        } else {
            eprintln!("Usage: commit-daily-logs.rs [--commit]");
            process::exit(1);
        }
    }

    // Snapshot the working copy so the full content lands in the operation
    // log before this script touches the file.
    jj(&["status"])?;

    let changed = jj(&["diff", "-r", "@", "--name-only"])?;
    let disallowed = changed
        .lines()
        .filter(|path| *path != DAILY_PATH)
        .collect::<Vec<_>>();
    if !disallowed.is_empty() {
        eprintln!("warning: working copy has changes outside {DAILY_PATH} (left uncommitted):");
        for path in disallowed {
            eprintln!("  {path}");
        }
    }

    let last_subject = jj(&[
        "log",
        "--no-graph",
        "-r",
        r#"latest(::@ & subject(glob:"log: ????-??-??"))"#,
        "-T",
        "description.first_line()",
    ])?;

    if last_subject.is_empty() {
        return Err(error("no prior \"log: YYYY-MM-DD\" commit found"));
    }
    let last_date = last_subject
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| error(format!("failed to parse commit subject: {last_subject}")))?
        .to_string();

    let base_content = jj(&["file", "show", "-r", "@-", DAILY_PATH])?;
    let current_content = fs::read_to_string(DAILY_PATH)?;

    let (base_lines, base_trailing_newline) = split_lines(&base_content);
    let (current_lines, current_trailing_newline) = split_lines(&current_content);

    validate_header(&base_lines, "@-:worklogs/daily.md")?;
    validate_header(&current_lines, DAILY_PATH)?;

    let headings = collect_headings(&current_lines)?;
    let last_pos = headings
        .iter()
        .position(|heading| heading.date == last_date)
        .ok_or_else(|| {
            error(format!(
                "last log date not found in {DAILY_PATH}: {last_date}"
            ))
        })?;

    if last_pos == 0 {
        eprintln!("no new dates to commit (last log: {last_date})");
        return Ok(());
    }

    let target_idx = last_pos - 1;
    let target_heading = &headings[target_idx];
    let block_start = target_heading.line_idx;
    let block_end = if let Some(next_heading) = headings.get(target_idx + 1) {
        next_heading.line_idx
    } else {
        current_lines.len()
    };
    let block_lines = &current_lines[block_start..block_end];

    if dry_run {
        println!("dry-run: would commit log: {}", target_heading.date);
        println!("dry-run: block lines {}..{}", block_start + 1, block_end);
        for line in block_lines {
            println!("{line}");
        }
        return Ok(());
    }

    println!("commit log: {}", target_heading.date);
    println!("block lines {}..{}", block_start + 1, block_end);
    for line in block_lines {
        println!("{line}");
    }

    let mut new_lines = Vec::new();
    new_lines.push(base_lines[0].clone());
    new_lines.push(base_lines[1].clone());
    new_lines.extend(block_lines.iter().cloned());
    new_lines.extend(base_lines.iter().skip(2).cloned());

    let mut new_content = new_lines.join("\n");
    let should_end_with_newline = base_trailing_newline || current_trailing_newline;
    if should_end_with_newline {
        new_content.push('\n');
    }

    // jj has no index: write the target content, let `jj commit` snapshot
    // and split it out by path, then restore the full file. Restore happens
    // before error propagation so the working copy never keeps a truncated
    // daily.md.
    fs::write(DAILY_PATH, &new_content)?;
    let message = format!("log: {}", target_heading.date);
    let commit_result = jj(&["commit", DAILY_PATH, "-m", &message]);
    fs::write(DAILY_PATH, &current_content)?;
    commit_result?;
    // Snapshot the restored content into the new working-copy commit.
    jj(&["status"])?;

    Ok(())
}
