#!/usr/bin/env bash
# Checkpoint: mirror the research docs (docs/ and PLAN.md) into engine/docs/research and push ONE commit to origin/main of tq-engine.
# Usage: docs/checkpoint.sh "docs: what this step did"
# Safe by design: refuses on secret-like strings, commits only docs/research by pathspec (never touches other staged files), never force-pushes.
set -euo pipefail
ROOT=/Users/thakur/Workspace/Trading
ENGINE="$ROOT/engine"
DEST="$ENGINE/docs/research"
msg="${1:?usage: checkpoint.sh \"type: message\"}"

if grep -rIlE --exclude=checkpoint.sh "(gho_|ghp_|github_pat_|sk-[A-Za-z0-9]{20}|AKIA[0-9A-Z]{12}|BEGIN (RSA|OPENSSH|PRIVATE) KEY)" "$ROOT/docs" "$ROOT/PLAN.md" 2>/dev/null; then
  echo "refusing to publish: secret-like string found in the files above" >&2; exit 1
fi
if find "$ROOT/docs" -type f \( -size +2M -o -name '*.gz' -o -name '*.zip' -o -name '*.pdf' \) | grep -q .; then
  echo "refusing to publish: large or binary file under docs/" >&2; exit 1
fi

mkdir -p "$DEST"
rsync -a --delete --exclude '__pycache__' --exclude '*.pyc' --exclude '.DS_Store' "$ROOT/docs/" "$DEST/"
cp "$ROOT/PLAN.md" "$DEST/PLAN.md"

cd "$ENGINE"
git add -- docs/research
if git diff --cached --quiet -- docs/research; then echo "nothing to checkpoint"; exit 0; fi
git commit -q -m "$msg" -- docs/research
git fetch -q origin
git push -q origin HEAD:main
git log --oneline -1
