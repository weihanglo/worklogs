# Monthly Summary Guide

Agent-oriented rules for generating monthly summaries in `monthly.md` from weekly entries in `weekly.md`.

## Overview

Aggregate weekly entries by theme, not chronologically.
Group related activities into cohesive narratives showing progress on broader goals.

## Workflow

### 1. Extract

* Read all weekly entries from target month in `weekly.md`
* Identify related work across multiple weeks
* Look for progression patterns (Started → Continued → Completed)

### 2. Group by Theme

* Combine related PRs/discussions under broader goals
* One clear theme per bullet point
* Split unrelated work even if from same week

**Valid themes:**

* Feature area: git handling, config system, build optimizations
* Major initiative: MSRV resolver, SQLite cache, gitoxide migration
* Ecosystem support: External project enablement, RFC implementation
* Stabilization work: Feature progression toward stable

### 3. Write

Follow narrative style from `.rules`.

In monthly summaries,
focus more on the big picture, business value, and the impact of the work.

## Examples

### ✅ Good: Multi-Week Initiative Grouped

```markdown
* Advanced SQLite-based index cache implementation to improve registry operation performance.
  Abstracted filesystem operations from on-disk cache, implemented core SQLite backend, and conducted performance benchmarks.
  This enables future parallel registry queries and reduces disk I/O for repeated operations —
  [rust-lang/cargo#13515](https://github.com/rust-lang/cargo/pull/13515)
  [rust-lang/cargo#13584](https://github.com/rust-lang/cargo/pull/13584)
  [rust-lang/cargo#6908](https://github.com/rust-lang/cargo/issues/6908)
```

Why good: Groups 3 weeks, leads with outcome, explains benefit, links original issue.

### ❌ Bad: Chronological List

```markdown
* Week 1: Started SQLite cache work
* Week 2: Continued SQLite implementation
* Week 3: Added benchmarks for cache
```

Why bad: Lists chronologically, no user impact, process-focused.

## Quick Reference

**What to include:**

* Major features and initiatives
* External project impact (name projects explicitly)
* RFC/FCP participation with ecosystem impact
* Security fixes and performance improvements

**What to exclude:**

* Minor cleanups, refactoring, test infrastructure
* Version bumps unless security critical
* Items that don't fit broader theme

**When uncertain:**

* Ask user about importance/priority
* Look at previous months for consistency
* Better to omit minor items than dilute key achievements
