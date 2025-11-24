#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <path-to-dot-file>" >&2
  exit 1
fi

DOT_FILE="$1"
if [ ! -f "$DOT_FILE" ]; then
  echo "dot file not found: $DOT_FILE" >&2
  exit 1
fi

OUTPUT="/tmp/$(basename "${DOT_FILE%.*}").png"

if ! command -v dot >/dev/null 2>&1; then
  echo "graphviz 'dot' command not found. Install graphviz to proceed." >&2
  exit 1
fi

dot -Tpng "$DOT_FILE" -o "$OUTPUT"

if command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$OUTPUT" >/dev/null 2>&1 &
else
  echo "Rendered PNG at $OUTPUT" >&2
fi
