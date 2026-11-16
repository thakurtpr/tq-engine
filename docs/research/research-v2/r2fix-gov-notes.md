# r2fix: governance side (tq-r2gov, tq r2 CLI, honesty close record)

Branch `hardening/r2fix`, fork scope: `crates/r2gov`, `crates/cli/src/r2_cmd.rs`, plus one additive
module in `crates/backtest` (`honesty::close_record`). Orchestrator rulings 2 to 7 and review
findings F-H1, F-H2, F-H5, F-H6, F-M4, F-M5, F-M6, F-L2 to F-L7 and survivors 9 to 14.
The prereg, the addendum, tags, thresholds and caps were not touched. No backtest was run, no real
ledger was opened or closed, `data-raw` was not modified (all raw-cut tests use tempdirs).

Tests: tq-r2gov lib 78 -> 138, tq-backtest `honesty` filter 93 -> 96 (+3 close_record), tq-cli 78 (unchanged).
`cargo clippy -p tq-r2gov -p tq-cli -p tq-backtest --all-targets -D warnings` is clean.
RED evidence (real runtime assertion failures before the fix) was observed for: G1, G2, caps pin, lightweight
tags, repo-level lock (5 tests), free-form close and cell storage, dev-GO binding (5 tests), batch
completeness, class validation, audit chain, allow-before-save. For the sealed store (encryption,
G3 offset, L4 symlink, read budget), the allowlist, G4 and the raw cut the implementation and tests were
written in the same step (the old API could not compile against the new tests), so no separate RED was
observed; the reviewer's own probe output (G3, G4) is the RED for those two.

## Finding status

| Finding | Status | Notes |
| --- | --- | --- |
| F-H1 pins | fixed | `gate::MASTER_SHA256` (0f36d2a5...f1733c) enforced in `tagged_master_hash`, so windows, split hash and sealing all follow the pinned text; `gate::CAPS_SHA256` (2f64af1d...ba79) enforced on `research/caps.json` at HEAD; `git diff code_git_sha..HEAD` over crates, Cargo.toml, Cargo.lock, caps.json must be empty (`HarnessChanged`); annotated tags only (`TagNotAnnotated`); sweep axes record `r2-master-tag-object` and `r2-candidate-tag-object` (annotated tag object ids). G1 and G2 are permanent tests. |
| F-H2 raw data | partially | `ingest_and_cut_raw` seals, verifies the sealed copy by decrypt+hash, then rewrites the raw compact CSVs (and `*.events.csv`) in place to the development end (atomic, all-or-nothing: any refusal or unparsable row aborts before any file is touched). Encrypted sealed store, outside repo and raw dir. NOT executed on the real data-raw (the coordinator runs `tq r2 seal-ingest` once per class). Residual: see below. |
| F-H5 dev-GO and close | fixed | see "formats". Dev-GO record carries `report_sha256` + `input_sha256`; the guard re-reads the committed report, requires verdict DEV-GO/DevGo for this candidate, and requires the stored cells to hash to the CLOSE merkle root. Close derives root and sr_var from the series. The old free-form `close_sweep` now always refuses (deprecated, same signature). |
| F-H6 close vs adjclose | fixed (partial for splits) | Addendum section 2 rule 3 is explicit, prereg 7.1 requires total return: non-index Yahoo series are ingested on `adjclose`, with the whole series multiplied by `close_L/adjclose_L` (L = last row <= dev end), which removes every dividend dated after the dev end from the level (returns identical). NSE indices stay on `close`. Splits dated after the dev end remain in `close` (level only; disclosed, the addendum says "where it can recompute"). Real-data finding: ADANIENT's adjclose is negative on 47 development-window rows (a Yahoo artefact), so total-return ingestion refuses it; use `--exclude ADANIENT` (a coordinator decision, record it) or supply a repaired file. |
| F-M4 batch protocol | fixed (a, b), partially (c) | (a) per-candidate results are escrowed encrypted (`SealedStore::escrow_result`) and readable only through `HoldoutGuard::release_results` after `finish_batch` and with one result per batch member. (b) `ack` refuses a batch that omits a GO-capable survivor (valid dev-GO and not a descendant of a burned candidate): `BatchIncomplete`. (c) read budget: one read per (candidate, symbol) via a persistent marker (`ReadBudget`) plus a 512-read token budget. Residual: `read_full` still returns plaintext to the caller; the control assumes the holdout runner is one reviewed binary. |
| F-M5 repo-level lock | fixed | lock file in the git common dir (`<common>/r2-research-v2-ledger.writer.lock`), shared by every worktree; every ledger write requires a registered writer (`<common>/r2-writer-worktree`: canonical worktree path + branch) matching the current worktree and branch (`NotRegisteredWriter`). Register once with `tq r2 register-writer`. No registration means no writes (fail closed). A same-user adversary can edit the registration file: procedural control; the committed anchor is the second line. |
| F-M6 class allowlist | fixed (CRYPTO B1 leak accepted) | `classmap::allows(class, series)`: NIFTYBEES/ETFs/indices only into IN-IDX (+ spot indices into IN-OPT), crypto/FX-for-crypto only into CRYPTO, plain tickers only into IN-EQ, commodities and foreign series nowhere. Cross-class ingestion is refused (`ClassMismatch`) before anything is written. The sealed-process benchmark engine of addendum rule 2 is NOT built: B1/B3 for crypto reach the evaluator as caller-supplied numbers; that leak (Indian buy-and-hold 2022 to 2026-10) is accepted and disclosed per addendum section 4. |
| F-L2 | fixed | `normalise_yahoo*` fail closed on non-finite, negative open, non-positive high/low/close (adjclose when used), high < low. A zero open stays allowed (known untrusted bar). Real-data note: only ADANIENT (adjclose), CL=F (negative oil) and GC=F (one high<low) trip this in the development window; the last two are in no class allowlist. |
| F-L3 | fixed | `row_date` parses RFC 3339 offsets and converts to the UTC date (G3). A time part that does not parse fails closed. |
| F-L4 | fixed | dev and sealed writes are tmp + fsync + rename; a pre-planted symlink at the file or at the class directory is refused. |
| F-L5 | fixed | the audit log is hash-chained (`prev`, `hash`, `AuditLog::tip()`); `read_all` rejects an edited, removed or reordered line; append is under an exclusive file lock. `with_ledger` now saves first and audits the allow afterwards; a failed save audits `ledger_save` deny and no allow. Residual: tail truncation is only detectable against a recorded tip (commit `tip()` with the ledger). |
| F-L6 | fixed | git runs with every inherited `GIT_*` variable removed and `-c core.fsmonitor=false`. |
| F-L7 | fixed | class names validated (`[A-Z0-9-]+`) in `ack`, `unseal`, `finish_batch`, `release_results`. |
| survivors 9, 11, 12, 13, 14 | killed by tests | 9: marker split-hash; 11: strategy-hash/tagged-prereg member check; 12: `truncate_csv` boundary (+ raw-cut boundary); 13: `valid_symbol` dot rules; 14: a refusal never saves. Not re-run under a mutation tool: the tests were written to fail under each described mutation. |
| survivor 10 | equivalent, documented | "is itself burned" is shadowed by the family test (same family) and cannot be observed separately. |

## Encryption (honest limits)

AES-256-GCM from RustCrypto `aes-gcm` 0.10.3 (MIT/Apache-2.0, NCC Group audit), fresh OS-entropy nonce per
file, AAD binds `class` and `symbol` (slot swaps fail), tamper fails the tag. The key is a 32-byte file outside
the repo and the raw-data directory, created `create_new` with mode 0400 (`tq r2 seal-ingest --create-key`),
loaded only if it is a regular file, exactly 0400, 32 bytes. Nothing in any brief or doc names a key path.
A process running as the same OS user can read a 0400 file, so this is NOT a barrier against a same-user
adversary: the real boundary is the OS (a separate user or machine holding the key). The control is
procedural plus defence in depth (nothing sealed is greppable or listable in plaintext, backups and repo
copies hold ciphertext only, the raw originals no longer contain post-dev rows).

## Residual risks

* The membership CSV `docs/data/nifty50_members_by_date.csv` (rows to 2026-09-30) and `data-ref/` are tracked in
  git and stay readable; git history is not rewritten. Working copies seen by strategy agents should use
  `DevReader::truncate_csv`. Documented leak.
* The raw cut is irreversible for the raw compact copies by design (re-derivable from upstream sources and the
  manifests); the sealed store is then the only local copy of post-dev rows, so back up the key file separately.
* Same-user adversary: key file, registration file, `chmod` and `git config` are all editable. OS-level
  separation is the real control.
* `code_git_sha` is now bound to HEAD for harness paths and caps are pinned, but a new reviewed build requires
  updating the two compiled-in hashes (that is the intent).
* `Pins::unpinned_caps` exists only under `cfg(test)`.
* Crypto B1/B3 benchmark leak accepted (addendum section 4); the sealed benchmark engine is not built.
* Splits after the dev end remain in the `close` level (disclosed).

## API changes (tq-r2gov, tq_backtest, tq r2)

Additions (backward compatible):
* `gate`: `MASTER_SHA256`, `CAPS_SHA256`, `CAPS_REL`, `Pins`, `verify_open_with`, `verify_marker_with`, `tag_object_id`.
  `verify_open`/`verify_marker` keep their signatures and use production pins. `check_code_sha` (crate-private) gained a `Pins` argument.
* `git::Git`: `tag_is_annotated`, `tag_object`, `changed_between`, `common_dir`, `toplevel`, `branch`.
* `registrar`: `register_writer`, `Registrar::{repo_lock_path, close_sweep_with_cells, read_cells, verify_cells}`, `Closed`,
  `AXIS_MASTER_TAG_OBJECT`, `AXIS_CANDIDATE_TAG_OBJECT`. `open_candidate_sweep` now also adds the two single-valued tag-object axes (N unchanged).
* `cells` module (per-cell series storage), `devgo` module (`DevGo`, `write_record`), `classmap`, `seal_crypto` (`SealKey`), `sealed_escrow` (`Released`).
* `sealed`: `SealedStore::{with_key, verify_sealed, escrow_result, has_result, read_results}`, `READS_PER_SYMBOL`, `TOKEN_READ_BUDGET`.
* `holdout`: `HoldoutGuard::release_results`; `DevGo` is re-exported from `devgo` with two new fields (`report_sha256`, `input_sha256`, serde default).
* `ingest`: `RawFormat::YahooTotalReturn`, `ingest_and_cut_raw`, `SealRun`. `datarules::normalise_yahoo_total_return`.
* `error::Code`: `PinMismatch, TagNotAnnotated, HarnessChanged, NotRegisteredWriter, ReadBudget, BatchIncomplete, BadCrypto, ClassMismatch`.
* `audit::AuditRecord`: `prev`, `hash` fields (serde default); `AuditLog::tip`.
* `tq_backtest::honesty::close_record::close_record(manifest, series) -> (merkle_root, sr_var)`; `sweep::{returns_hash, root_of}` are now `pub(crate)`.
* CLI: `tq r2 register-writer`, `tq r2 dev-go`.

Behaviour changes (may need reconciling):
* `Registrar::close_sweep(sweep_id, root, cells, sr_var)` is deprecated and ALWAYS refuses (`BadInput`). Use `close_sweep_with_cells`.
  CLI `close-sweep` now takes `--sweep-id` and `--cells-dir` (no `--merkle-root/--cells/--sr-var`).
* `tq r2 seal-ingest` requires `--key-file` (use `--create-key` once), `--yahoo` now means the total-return basis, new `--exclude`; it cuts the raw directory in place.
* Every ledger write requires `tq r2 register-writer` first, and runs only from the registered worktree and branch. The writer lock moved to the common git dir.
* `ingest_dir` now rejects series outside the class allowlist and handles `*.events.csv` as dated CSVs (previously a hard error under the Yahoo format).
* The sealed layout changed: `<root>/sealed/<class>/<symbol>.enc` (was mode-000 plaintext `.csv`); the mode-000 directory dance is gone.
* `HoldoutGuard::ack` additionally refuses an incomplete batch. `finish_batch` is unchanged.
* `Repo::tag` (test util) now creates annotated tags.

## Formats the evaluator (r2stats) and the runner (r2run) rely on

* Per-cell returns: `research/cells/<sweep_id>/<index:05>.f64`, raw little-endian `f64`, cell-index order of
  `tq_backtest::honesty::sweep::enumerate_cells(manifest)`. Written once by `close_sweep_with_cells` BEFORE the CLOSE entry.
* CLOSE record: `tq_backtest::honesty::close_record::close_record(&manifest, &series) -> (merkle_root_hex, sr_var)`.
  Leaf = `cell|<index>|<k=v,...>|sr=<hex bits or none>|ret=<sha256 of the f64 LE bytes>`; root = the honesty merkle
  (leaf 0x00, node 0x01, odd node promoted). Compare with `ledger.close_root(sweep_id)`; the sr_var bits are in the CLOSE entry
  (`sr_var_bits`). r2stats depends on tq-backtest, so it can verify without r2gov. `Registrar::verify_cells` does the same.
* Dev-GO: `research/dev-go/<id>.json` = `{"candidate_id","sweep_id","verdict":"GO","report_sha256","input_sha256"}` and
  `research/dev-go/<id>.report.json` (the evaluator's JSON, byte-identical to what `report_sha256` covers, committed).
  Required report fields: `candidate_id` equal to the id, `verdict` one of `"DEV-GO"` or `"DevGo"` (the serde name of the renamed verdict),
  optional `input_sha256` (must equal the record's). Suggestion for r2stats: emit `input_sha256` (sha256 of the canonical evaluator input JSON) in `GateReport`.
* Tag axes in sweep manifests: `r2-master-tag-object`, `r2-candidate-tag-object` (40-hex annotated tag object ids).
* Merge note: `Cargo.lock` gained `aes-gcm` and its RustCrypto dependencies (r2gov only); expect a trivial lock merge with the r2stats branch.
