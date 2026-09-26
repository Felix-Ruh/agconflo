# Claude Code notes for Agconflo

**Read [AGENTS.md](AGENTS.md) first.** It is the working mode and applies here in full: the cycle,
the plan format, the two stages of a behaviour change, the diff review before every commit, the
three kinds of test, commits and branches, reporting, and what to stop and ask about. The README is
the mechanical reference — setup, gates, branch protection, the metamodel.

This file covers only what is specific to running as Claude Code here.

## This project keeps no agent memory

Everything needed is in the repository: `README.md`, `AGENTS.md`, and `docs/` for the requirements
themselves. Do not create memory files for this project, and do not treat anything outside the
repository as authoritative. When something is worth keeping, it belongs in one of those files, in
a commit that can be reviewed.

## Resuming after a compaction

A compaction summary is one agent's lossy account of a session, written under pressure. It is a
pointer, never the record. Rebuild context in this order, and the order matters: reading the
transcript first inherits the previous agent's framing, its mistakes included.

1. **The repository.** `README.md`, then `AGENTS.md`, then the files the next task touches — whole
   files, not greps.
2. **Current state.** `git status`, `git log --oneline -5`,
   `git rev-list --left-right --count origin/main...main`, and the gates if anything looks unclear.
3. **The transcript, last.**

### Reading the transcript

It lives at `~/.claude/projects/<project-slug>/<session-uuid>.jsonl`, and the current session's
uuid is in the scratchpad path given in the environment — no guessing from modification times. It
keeps everything the terminal no longer renders, because compaction drops rows from the model's
context, not from the file.

Four things make a naive extraction wrong, all of them measured:

- **Filter per content block, not per row.** Prose written before a tool call sits in the *same*
  assistant row as the `tool_use` block.
- **A `user` row carrying `toolUseResult` is a tool result**, not something a person typed. A row
  with `isCompactSummary` is the summary, not their words either. `isSidechain: true` is subagent
  traffic.
- **A message sent while a turn is running is not a `user` row at all.** It arrives as
  `type: "attachment"` with `attachment.type == "queued_command"`, its text under
  `attachment.prompt`, and `attachment.origin.kind == "human"`. Roughly half of everything said
  arrives this way and it carries real instructions, so an extraction that misses it looks complete
  while lacking whole requirements. Dedupe on `source_uuid`: one queued message is re-surfaced
  beside several tool results.
- **The file can hold the same rows twice.** Whole stretches are re-appended with identical
  `uuid`s, which makes counts of messages and of compaction boundaries alike come out wrong. Keep a
  set of seen `uuid`s and skip repeats.

Extract everything to the scratchpad and read selectively from there: every human message in full,
used as an index into the rest, and the assistant's own prose only for the last few turns — the
last direction set, the last plan approved, the last report. Set `PYTHONIOENCODING=utf-8` when
extracting with Python; the prose is full of dashes and arrows.

Then report how far back the transcript goes and whether it is complete, and correct anything the
repository asserts that the files contradict.

## Working with the person asking

- **Anything requiring action on their part goes in the final message, or in a question.** Text
  written between tool calls is not read.
- **Use the question tool only for what is genuinely theirs to decide** — pushing, opening or
  merging; a value only they have; a confirmation to click; a preference this repository does not
  record. A choice the files, a query or an experiment can settle is yours to settle and report,
  and putting it to them instead hands back the work you were asked to do.
- **Do not start background pollers or scheduled wake-ups** to wait on something long. Hand it
  over, say what to look for, and stop.
- **Long or expensive commands get interrupted.** Consider the cost before running one.

## Shell and tooling

- The shell rules — POSIX `sh` for the hook and the scripts, `ubc` invoked by path — are in the
  README, under "Prerequisites" and "The commit hook". They apply unchanged.
- In a Claude Code cloud session, `.claude/hooks/session-start.sh` does the README's setup, starts
  the Docker daemon, pulls the images the tool tests pin and builds the workspace before the session
  begins. What it could not do it names on standard error; a local clone skips it.
- **Exit 126 from a binary is group policy, not a bad download.** `[ -x ]` returns true for a
  blocked binary, so probe by running it — and ask rather than working around a block.
- Write commit messages through a Bash heredoc. A PowerShell here-string mangles the subject line.
- Keep throwaway scripts, probes and scratch clones in the scratchpad, never in the repository.
- Files here are LF, pinned in `.gitattributes`. Writing from Windows tooling can produce CRLF, so
  check with `git ls-files --eol` before committing.
- Independent tool calls go in one message, in parallel.
