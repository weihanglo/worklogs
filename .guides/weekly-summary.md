# Weekly Summary Guide

Rules for generating weekly summaries in `weekly.md` from daily PR logs in `daily.md`.

## Week Boundaries - CRITICAL

* Header = Monday when summary is written (week already ended)
* Coverage = Previous 7 days (Monday-Sunday of prior week)
* Example: `## 2024-01-08` covers Jan 1-7, 2024

## Workflow

### 1. Extract

- Extract all PR/issue links from daily.md for the 7-day coverage period
- Skip `rust-lang/rust` PRs titled "Update cargo" (just submodule bumps)

### 2. Classify

For each PR, use `gh pr view <PR-URL>` (add `--comments` or `--diff` if needed):

- Determine: WHAT changed, WHY it matters, WHO benefits
- Classify: **Include** (full write-up), **TODO** (prefixed), or **Skip**

**Include with full write-up:**

- User says it's important
- Unblocks external projects (e.g., Eclipse Zenoh)
- Design discussions, RFC/FCP reviews
- Fixes user-facing bugs or security issues
- Beta backports (shows urgency)

**Include with TODO prefix:**

- Version bumps, changelog updates (unless security critical)
- Refactoring without clear user benefit
- Test infrastructure (unless critical)
- Minor cleanups, typo fixes

**Skip:**

- Cargo submodule updates in rust-lang/rust
- Work already documented elsewhere in the week

**If uncertain:** Mark TODO and ask user.

### 3. Group

Group PRs by theme (NOT chronologically):
- Feature area: git, config, build, package ops
- User impact: DX, performance, debugger support
- Initiative: RFC impl, stabilization, ecosystem support
- Multi-week work: "Started", "Continued", "Completed"

### 4. Write

- Lead with outcome: "Fixed", "Enabled", "Unblocked" (NOT "Worked on", "Reviewed")
- Explain: WHO benefits, WHAT it enables, WHY it matters
- Link: Include issues (via "Fixes #XXXX"), RFCs, Project Goals
- Multi-week: Use "Started" (full detail), "Continued" (less detail), "Completed" (impact)

## Classification Decision Tree

```
Is it cargo submodule update in rust-lang/rust?
└─ YES → Skip

Can you infer clear user impact?
├─ YES → Include (full write-up)
└─ NO ↓

Is it refactoring/test/version-bump/cleanup?
├─ YES → Include (TODO prefix)
└─ NO → Mark TODO, ask user
```

## Examples

### ✅ Full Write-Up: Clear External Impact
```markdown
* Fixed `cargo update --precise` to accept arbitrary Git revisions including short SHAs, tags, and branch names for Git dependencies.
  Previously, libgit2's zero-padding of short SHAs caused lookup failures, preventing users from pinning dependencies to specific commits by tag or short hash.
  This unblocked Eclipse Foundation's Zenoh project where plugins need exact commit matching with the hosting binary to avoid ABI incompatibility —
  [rust-lang/cargo#13250](https://github.com/rust-lang/cargo/pull/13250)
  [rust-lang/cargo#13188](https://github.com/rust-lang/cargo/issues/13188)
```

Why good: Outcome-focused verb ("Fixed"), explains broken behavior, shows user impact, names external beneficiary, includes issue link.

### ✅ TODO: Internal Refactoring
```markdown
* TODO: Refactored string joining utilities to use itertools —
  [rust-lang/cargo#13275](https://github.com/rust-lang/cargo/pull/13275)
```

Why TODO: Pure refactoring without clear user benefit.

## Quick Reference

**Finding context:**
1. Start: `gh pr view <URL>` → title + description
2. If unclear: `gh pr view <URL> --comments` → discussion
3. Still unclear: `gh pr diff <URL>` → code changes
4. Check "Fixes #XXXX" → original user problem

**User feedback patterns:**
- User says "X is important" → Full write-up
- User says "Y not important" → TODO or omit
- Better to mark TODO than write weak rationale
