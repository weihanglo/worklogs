#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
package.edition = "2024"

[dependencies]
# no dependencies needed
---

//! # Worklog Statistics Tool
//!
//! A parser for the half-broken worklog format.
//!
//! ## Usage
//!
//! ```console
//! # Current year stats
//! ./worklog-stats.rs worklogs/daily.md
//!
//! # Specific date range
//! ./worklog-stats.rs worklogs/daily.md --from 2025-01 --to 2025-03
//!
//! # Export to CSV
//! ./worklog-stats.rs worklogs/daily.md --csv output.csv
//!
//! # Combine options
//! ./worklog-stats.rs worklogs/daily.md --from 2025 --to 2025 --csv output.csv
//! ```
//!
//! ## Date Formats
//!
//! - `2025` — full year (2025-01-01 to 2025-12-31)
//! - `2025-07` — full month (2025-07-01 to 2025-07-31)
//! - `2025-07-15` — specific date

use std::collections::{HashMap, HashSet};
use std::env;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write as _;
use std::process;
use std::str::FromStr;

type BoxResult<T> = Result<T, Box<dyn error::Error>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    #[default]
    IssueTriage,
    RfcReview,
    FcpReview,
    PrReview,
    PrSubmission,
    Discussion,
    Research,
}

impl FromStr for Kind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "Issue triages" => Kind::IssueTriage,
            "RFC reviews" => Kind::RfcReview,
            "FCP reviews" => Kind::FcpReview,
            "PR reviews" => Kind::PrReview,
            "PR submissions" => Kind::PrSubmission,
            "Discussions" => Kind::Discussion,
            "Researches" => Kind::Research,
            _ => return Err(format!("Unrecognizable kind `{s}`")),
        };
        Ok(kind)
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Closed,
    Commented,
    Created,
    Merged,
    Tracked,
    Updated,
    Mentored,
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let action = match s {
            "Closed" => Action::Closed,
            "Commented" => Action::Commented,
            "Created" => Action::Created,
            "Merged" => Action::Merged,
            "Tracked" => Action::Tracked,
            "Updated" => Action::Updated,
            "Mentored" => Action::Mentored,
            _ => return Err(format!("Unrecognizable action `{s}`")),
        };
        Ok(action)
    }
}

#[derive(Debug, Clone)]
struct Record {
    date: String,
    kind: Kind,
    action: Action,
    url: String,
    canonical_url: String,
}

impl Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            date,
            kind,
            action,
            url,
            canonical_url,
        } = self;
        write!(f, "{date},{kind:?},{action:?},{url},{canonical_url}")
    }
}

#[derive(Debug)]
struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.0)
    }
}

impl error::Error for Error {}

fn error(msg: String) -> Box<Error> {
    Box::new(Error(msg))
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

/// Check if a date string (YYYY-MM-DD) is within a range (inclusive).
fn date_in_range(date: &str, start: &str, end: &str) -> bool {
    date >= start && date <= end
}

/// Parse a date argument, supporting shortcuts like "2025-01" -> "2025-01-01"
fn normalize_date(s: &str, is_end: bool) -> Option<String> {
    let parts: Vec<&str> = s.split('-').collect();
    match parts.len() {
        // YYYY -> YYYY-01-01 or YYYY-12-31
        1 if parts[0].len() == 4 => {
            if is_end {
                Some(format!("{}-12-31", parts[0]))
            } else {
                Some(format!("{}-01-01", parts[0]))
            }
        }
        // YYYY-MM -> YYYY-MM-01 or YYYY-MM-{last day}
        2 if parts[0].len() == 4 && parts[1].len() == 2 => {
            if is_end {
                let last_day = match parts[1] {
                    "02" => "28",
                    "04" | "06" | "09" | "11" => "30",
                    _ => "31",
                };
                Some(format!("{}-{}-{}", parts[0], parts[1], last_day))
            } else {
                Some(format!("{}-{}-01", parts[0], parts[1]))
            }
        }
        // YYYY-MM-DD
        3 if parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2 => {
            Some(s.to_string())
        }
        _ => None,
    }
}

fn parse_item(date: &str, kind: Kind, text: &str) -> BoxResult<Record> {
    let (action, url) = match text
        .trim_start_matches(&['-', ' ', '*'])
        .trim()
        .split_once(' ')
    {
        Some(s) => s,
        None => return Err(error(format!("failed to parse `{text}`"))),
    };
    let action = action.parse::<Action>()?;
    let canonical_url = url.rsplit_once('#').map(|x| x.0).unwrap_or(url).into();
    let record = Record {
        date: date.to_string(),
        kind,
        action,
        canonical_url,
        url: url.into(),
    };
    Ok(record)
}

/// Parse worklog file and return all records (lenient mode for stats).
fn parse_worklog(path: &str) -> BoxResult<Vec<Record>> {
    let input = File::open(path)?;
    let mut records = Vec::new();
    let mut date = String::new();
    let mut kind = Kind::default();

    for (_lnr, line) in BufReader::new(input).lines().skip(1).enumerate() {
        let line = line?;
        if line.len() < 2 {
            continue;
        }
        match line.split_at(2) {
            ("", _) => continue,
            (prefix, "") if !prefix.is_empty() => continue,
            ("##", text) => {
                let next_date = text.trim();
                if is_date(next_date) {
                    date = next_date.to_string();
                }
                continue;
            }
            ("- " | "* ", text) => {
                if let Ok(k) = text.trim().parse() {
                    kind = k;
                }
                continue;
            }
            ("  ", text) => {
                if let Ok(record) = parse_item(&date, kind, text) {
                    records.push(record);
                }
            }
            _ => continue,
        };
    }

    Ok(records)
}

fn extract_repo(url: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }
    None
}

#[derive(Default)]
struct Args {
    input: Option<String>,
    from: Option<String>,
    to: Option<String>,
    csv: Option<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    let mut argv: Vec<String> = env::args().skip(1).collect();

    // Parse flags
    while !argv.is_empty() {
        let arg = &argv[0];
        if arg == "--from" {
            argv.remove(0);
            args.from = Some(argv.remove(0));
        } else if arg == "--to" {
            argv.remove(0);
            args.to = Some(argv.remove(0));
        } else if arg == "--csv" {
            argv.remove(0);
            args.csv = Some(argv.remove(0));
        } else if arg.starts_with('-') {
            return Err(format!("unknown flag: {arg}"));
        } else {
            args.input = Some(argv.remove(0));
        }
    }

    Ok(args)
}

fn current_year() -> String {
    if let Ok(date) = std::process::Command::new("date").arg("+%Y").output() {
        if let Ok(year_str) = std::str::from_utf8(&date.stdout) {
            return year_str.trim().to_string();
        }
    }
    "2025".to_string()
}

fn print_usage() {
    eprintln!(
        r#"Worklog Statistics Tool

Usage:
    worklog-stats.rs <input.md> [--from DATE] [--to DATE] [--csv OUTPUT]

Options:
    --from DATE    Start date (default: current year start)
    --to DATE      End date (default: current year end)
    --csv OUTPUT   Export to CSV file instead of printing stats

Date formats:
    2025           Full year (2025-01-01 to 2025-12-31)
    2025-07        Full month (2025-07-01 to 2025-07-31)
    2025-07-15     Specific date

Examples:
    ./worklog-stats.rs worklogs/daily.md
    ./worklog-stats.rs worklogs/daily.md --from 2025-01 --to 2025-06
    ./worklog-stats.rs worklogs/daily.md --csv output.csv
    ./worklog-stats.rs worklogs/daily.md --from 2025 --csv 2025.csv"#
    );
}

fn main() -> BoxResult<()> {
    let args = parse_args().map_err(|e| error(e))?;

    let input = match args.input {
        Some(p) => p,
        None => {
            print_usage();
            process::exit(1);
        }
    };

    // Default to current year
    let year = current_year();
    let from_str = args.from.unwrap_or_else(|| year.clone());
    let to_str = args.to.unwrap_or_else(|| year.clone());

    let start_date = normalize_date(&from_str, false)
        .ok_or_else(|| error(format!("invalid date: {from_str}")))?;
    let end_date =
        normalize_date(&to_str, true).ok_or_else(|| error(format!("invalid date: {to_str}")))?;

    eprintln!("Parsing: {input}");
    eprintln!("Range: {start_date} to {end_date}");

    let records = parse_worklog(&input)?;

    // Filter by date range
    let filtered: Vec<_> = records
        .iter()
        .filter(|r| date_in_range(&r.date, &start_date, &end_date))
        .collect();

    if filtered.is_empty() {
        eprintln!("No entries found in range");
        process::exit(1);
    }

    // CSV mode
    if let Some(csv_path) = args.csv {
        let mut output = File::create(&csv_path)?;
        for record in &filtered {
            writeln!(output, "{record}")?;
        }
        eprintln!("Written: {csv_path} ({} records)", filtered.len());
        return Ok(());
    }

    // Stats mode
    let mut days: HashSet<&str> = HashSet::new();
    let mut contributions: HashSet<&str> = HashSet::new();
    let mut reviews: HashSet<&str> = HashSet::new();
    let mut triage: HashSet<&str> = HashSet::new();
    let mut fcp: HashSet<&str> = HashSet::new();
    let mut rfc: HashSet<&str> = HashSet::new();
    let mut repos: HashSet<String> = HashSet::new();
    let mut repo_counts: HashMap<String, usize> = HashMap::new();

    for record in &filtered {
        days.insert(&record.date);

        if let Some(repo) = extract_repo(&record.canonical_url) {
            *repo_counts.entry(repo.clone()).or_default() += 1;
            if record.kind == Kind::PrSubmission {
                repos.insert(repo);
            }
        }

        match record.kind {
            Kind::PrSubmission => {
                contributions.insert(&record.canonical_url);
            }
            Kind::PrReview => {
                reviews.insert(&record.canonical_url);
            }
            Kind::IssueTriage => {
                triage.insert(&record.canonical_url);
            }
            Kind::FcpReview => {
                fcp.insert(&record.canonical_url);
            }
            Kind::RfcReview => {
                rfc.insert(&record.canonical_url);
            }
            _ => {}
        }
    }

    // Print stats
    println!("* {} days logged", days.len());
    println!("* {} PRs authored", contributions.len());
    println!("* {} PRs reviewed", reviews.len());
    println!("* {} issues triaged", triage.len());
    if !fcp.is_empty() || !rfc.is_empty() {
        println!("* {} FCPs, {} RFCs reviewed", fcp.len(), rfc.len());
    }
    println!("* Contributed to {} repositories", repos.len());

    // Top repos
    if !repo_counts.is_empty() {
        let mut repo_list: Vec<_> = repo_counts.iter().collect();
        repo_list.sort_by(|a, b| b.1.cmp(a.1));
        println!("\nTop repositories:");
        for (repo, count) in repo_list.iter().take(5) {
            println!("  * {repo}: {count}");
        }
    }

    Ok(())
}
