# Gotchas and lessons

One line each. Add when you hit a trap, a non-obvious behavior, or a failed approach worth remembering.

- 2026-11-06 build: always set CARGO_TARGET_DIR=/Users/thakur/Workspace/Trading/shared-target; per-worktree target dirs filled the disk (2-8 GB each).
- 2026-11-06 hooks: a "Fact-Forcing Gate" hook blocks first Bash/Write/Edit calls until the facts (request, purpose, callers) are stated, then the retry succeeds.
- 2026-11-06 data: free sources 403 (incometaxindia.gov.in); archives.nseindia.com serves NSE circular PDFs; browser-spoofed curl to NSE is blocked, do not work around it.
- 2026-11-07 benchmark: journal.csv has a header row; date checks must skip it (tail -n +2) and use `!=` not `>` in zsh.
- 2026-11-07 tax: all equity/crypto tax rates are secondary-sourced and unverified except F&O STT, NSE charges and lot sizes.
