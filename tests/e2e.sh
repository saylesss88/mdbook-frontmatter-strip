#!/usr/bin/env bash
# Build tests/book with whatever `mdbook` and `mdbook-frontmatter-strip` are on
# PATH, then check the preprocessed Markdown output.
#
# Locally:
#   cargo build --release
#   PATH="$PWD/target/release:$PATH" tests/e2e.sh
set -euo pipefail

cd "$(dirname "$0")/book"
rm -rf book
mdbook --version
mdbook build

out=book/markdown
fail=0

# Each chapter should now start with its body line.
for name in intro fenced nested unfenced; do
    file="$out/$name.md"
    want="${name^^} BODY"
    first=$(head -n1 "$file" | tr -d '\r')
    if [[ "$first" != "$want" ]]; then
        echo "FAIL: $file starts with '$first', expected '$want'"
        fail=1
    fi
done

# A chapter that opens with a horizontal rule must be left alone.
if [[ "$(head -n1 "$out/rule.md")" != "---" ]]; then
    echo "FAIL: rule.md lost its leading horizontal rule"
    fail=1
fi

# No frontmatter should reach any renderer's output, including the HTML
# search index. Match the fixture's own titles so mdBook's bundled JS
# can't cause false positives.
pattern="title: (Intro|Fenced|Nested|Unfenced)"
if grep -rqE "$pattern" book/; then
    echo "FAIL: frontmatter found in output:"
    grep -rlE "$pattern" book/
    fail=1
fi

if [[ $fail -eq 0 ]]; then
    echo "OK: all chapters stripped correctly"
fi
exit $fail
