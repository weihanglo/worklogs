# Yearly Summary Guide

Agent-oriented rules for generating yearly (H1/H2) summaries in `yearly.md` from monthly entries in `monthly.md`.

## Overview

Yearly summaries differ from monthly summaries in abstraction level and narrative style.
They tell a cohesive story of the half-year rather than just aggregating monthly entries.

**Key differences from monthly:**

| Aspect | Monthly | Yearly |
|--------|---------|--------|
| Abstraction | PR/feature level | Strategic theme level |
| Narrative | Bullet items | Story with overview |
| Detail | Individual PRs | Aggregated outcomes |
| Audience | Self-reference | Manager, external readers |

## Workflow

### 0. Generate Statistics

Run the statistics extraction script to get metrics for the summary header:

```bash
# H1 stats (Jan-Jun)
./worklog-stats.rs worklogs/daily.md --from 2025-01 --to 2025-06

# H2 stats (Jul-Dec)
./worklog-stats.rs worklogs/daily.md --from 2025-07 --to 2025-12
```

Output looks like:

```markdown
## 2025 H1

* 141 days logged
* 96 PRs authored
* 176 PRs reviewed
* 172 issues triaged
* 6 FCPs, 2 RFCs reviewed
* Contributed to 8 repositories
```

Place this block at the top of each H1/H2 section in `yearly.md`, before the narrative overview.

**Metrics explained:**

* **Days logged** — unique dates with entries in daily.md
* **PRs authored** — unique URLs under "PR submissions"
* **PRs reviewed** — unique URLs under "PR reviews"
* **Issues triaged** — unique URLs under "Issue triages"
* **FCPs/RFCs** — unique URLs under those categories
* **Repositories** — unique repos you contributed code to

Note: "Merged" counts require GitHub API queries (see Kobzol's script).
The worklog-based counts track activity, not outcomes.

### 1. Extract

* Read all monthly entries for target half-year (H1: Jan-Jun, H2: Jul-Dec)
* Identify major initiatives spanning multiple months
* Note Rust Project Goals ownership/championship
* Track stabilizations, RFC work, cross-team efforts

### 2. Categorize by Strategic Theme

Don't copy monthly categories. Choose categories that reflect actual work patterns.

**Example categories (adapt as needed):**

* Rust Project Goals — official goals where user is owner/champion
* Stabilization — features driven toward stable
* Performance & Build Speed — measurable improvements
* Bug Fixes & Regression Triage — maintainer reliability work
* User Experience & Linting — user-facing improvements
* Supply Chain & Security — provenance, auditing, publishing
* Cross-team & Infrastructure — work affecting multiple teams/repos
* Community Leadership — mentoring, contributor guidance

### 3. Write Strategic Overview

Start with 5-8 line overview summarizing the half-year:

* What were the 2-3 major focuses?
* Key outcomes with numbers if available (e.g., "10x speedup")
* Who was unblocked (name companies/projects)
* Other highlights (mentoring, cross-team work)

### 4. Write Section Content

**Accuracy of role:**

* "Prototyped as goal owner" — you designed and implemented
* "Served as goal champion" — you provided feedback, not primary implementer
* "Drove" — you pushed the process forward
* "Shepherded" — you guided through FCP/stabilization process
* "Reviewed" — you provided feedback on others' work
* "Attempted" / "Drafted" — work not yet landed

**Clarify who wants what:**

* "$WORK wants X" — internal business need
* "Requested by [Company/Project]" — external demand
* "Unblocks [specific users]" — name beneficiaries

**Connect related work:**

* Show how pieces fit together (e.g., "checksum freshness complements new build directory layout")
* Reference Rust Project Goals when applicable

### 5. Review and Adjust

**Ask user about:**

* Items to emphasize or de-emphasize
* Role accuracy (reviewer vs implementer)
* Missing context for proposals/drafts
* Company/project names that want features

**Don't over-simplify:**

* User may want to keep detail for emphasis
* Ask before condensing bullet lists into prose
* Preserve specific beneficiary names

## What to Include

* Rust Project Goal ownership and championship
* Major stabilizations with user impact
* Performance improvements with numbers
* Cross-team collaboration and infrastructure work
* Mentoring (can be condensed to one bullet)
* Proposals and drafts (mark as such)

## What to Exclude

* Minor refactoring, test infrastructure
* Items already covered under broader themes
* Excessive PR-level detail (belongs in monthly)

## Formatting

**Non-exhaustive lists:**

When listing items that aren't exhaustive, use "including" or similar words
to signal there's more not mentioned:

```markdown
* Shepherded multiple FCP/stabilization processes, including:
  * `pubtime` field in registry index — unblocks dependency freshness
  * TOML 1.1 parse support — newline literals without breaking MSRV

* Platform compatibility fixes, including:
  migrated Cargo to libstd flock, released `jobserver@0.1.34`,
  and fixed `cargo package` timestamp issues.

* Mentored contributors on various improvements,
  including Git submodule caching, file locking, and `-Zbindeps` bug fix.
```

This avoids implying the list is complete when items were omitted for brevity.

**Sub-lists for related items:**

Use sub-lists when items share the same type of contribution (e.g., all FCPs).

**Combining regression triage:**

```markdown
* Triaged multiple beta regressions and coordinated backports,
  including curl-sys SSL errors on FreeBSD, flaky Windows CI tests,
  and concurrent `cargo check` locking bugs.
```

Don't need full sub-list for every regression unless individually significant.

## Context Fetching

For GitHub links, use `gh` CLI to understand context:

```bash
gh pr view <URL> --json title,body
gh issue view <URL> --json title,body
gh pr view <URL> --comments  # if discussion matters
```

This helps:

* Understand connections between work items
* Verify role (implementer vs reviewer)
* Find who requested/benefits from features

## Examples

### ✅ Good: Strategic Overview

```markdown
H2 centered on two official Rust Project Goals
(Build Analysis, Build Directory Layout)
and driving long-awaited stabilizations that unblock enterprise users.
Key outcomes include 10x rustdoc speedup,
`-Zconfig-include` stabilization for TockOS and embedded developers,
and laying groundwork for safer multi-crate publishing.
Also mentored multiple first-time contributors
and led cross-team infrastructure discussions.
```

Why good: Names specific goals, quantifies outcomes, names beneficiaries, shows breadth.

### ✅ Good: Role Clarity

```markdown
* Prototyped [Cargo Build Analysis](...) as goal owner and implementer.
```

```markdown
* Served as goal champion for [Cargo Build Directory Layout](...),
  providing feedback on design decisions.
```

Why good: Clear distinction between owner/implementer vs champion/reviewer.

### ✅ Good: Connecting Work

```markdown
* Unblocked `-Zchecksum-hash-algorithm` by fixing rustc span tracking.
  Checksum-based freshness complements the new build directory layout,
  avoiding spurious rebuilds when mtime changes but content stays the same.
```

Why good: Shows how separate items work together.

### ❌ Bad: Monthly Copy-Paste

```markdown
## 2025 H2

### July
* Did X
* Did Y

### August
* Did Z
```

Why bad: Just reorganized monthly, no strategic narrative.

### ❌ Bad: Overstating Role

```markdown
* Championed and drove implementation of build directory layout...
```

Why bad: If you only reviewed, don't claim implementation credit.

## Quick Reference

**Before writing:**

1. Read all 6 months of monthly entries
2. Identify 2-3 major themes/focuses
3. Check Rust Project Goals for ownership roles
4. Note what landed vs what's still draft/proposed

**During writing:**

1. Start with strategic overview
2. Group by theme, not time
3. Clarify role for each item
4. Name beneficiaries explicitly
5. Connect related work

**After draft:**

1. Ask user about emphasis/de-emphasis
2. Verify role accuracy
3. Check draft/proposal status
4. Don't over-condense without permission
