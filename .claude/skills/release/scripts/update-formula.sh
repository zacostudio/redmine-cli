#!/usr/bin/env bash
# homebrew-redmine Formula 자동 갱신 스크립트
# usage: update-formula.sh X.Y.Z
# - tap 저장소를 /tmp/homebrew-redmine 에 clone(또는 pull)
# - GitHub Release의 .sha256 파일 3개를 받아 Formula/redmine.rb 갱신
# - 변경 사항은 diff로 보여주고, 커밋/푸시는 호출자에게 맡긴다 (--push 미지원)

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 X.Y.Z" >&2
  exit 2
fi

VERSION="$1"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: version must look like X.Y.Z (got '$VERSION')" >&2
  exit 2
fi

TAG="v${VERSION}"
TAP_DIR="${TAP_DIR:-/tmp/homebrew-redmine}"
RELEASE_REPO="zacostudio/redmine-cli"
TAP_REPO="zacostudio/homebrew-redmine"
FORMULA_PATH="${TAP_DIR}/Formula/redmine.rb"
BASE_URL="https://github.com/${RELEASE_REPO}/releases/download/${TAG}"

TARGETS=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  x86_64-unknown-linux-gnu
)

echo "==> ensure tap clone at ${TAP_DIR}"
if [[ -d "${TAP_DIR}/.git" ]]; then
  git -C "${TAP_DIR}" fetch origin
  git -C "${TAP_DIR}" checkout master
  git -C "${TAP_DIR}" pull --ff-only origin master
else
  gh repo clone "${TAP_REPO}" "${TAP_DIR}"
fi

if [[ ! -f "${FORMULA_PATH}" ]]; then
  echo "error: ${FORMULA_PATH} not found" >&2
  exit 1
fi

echo "==> fetch sha256 sidecar files for ${TAG}"
fetch_sha() {
  local target="$1"
  local url="${BASE_URL}/redmine-${TAG}-${target}.tar.gz.sha256"
  local line hex
  line="$(curl -fsSL "$url")"
  hex="${line%% *}"
  if [[ ! "$hex" =~ ^[0-9a-f]{64}$ ]]; then
    echo "error: invalid sha256 from $url -> '$line'" >&2
    exit 1
  fi
  echo "$hex"
}

SHA_ARM_MAC="$(fetch_sha aarch64-apple-darwin)"
SHA_INTEL_MAC="$(fetch_sha x86_64-apple-darwin)"
SHA_LINUX="$(fetch_sha x86_64-unknown-linux-gnu)"

echo "    aarch64-apple-darwin: ${SHA_ARM_MAC}"
echo "    x86_64-apple-darwin:  ${SHA_INTEL_MAC}"
echo "    x86_64-unknown-linux: ${SHA_LINUX}"

echo "==> rewrite Formula/redmine.rb"

python3 - "$FORMULA_PATH" "$VERSION" \
  "$SHA_ARM_MAC" \
  "$SHA_INTEL_MAC" \
  "$SHA_LINUX" <<'PY'
import re, sys, pathlib

path = pathlib.Path(sys.argv[1])
version, arm_mac, intel_mac, linux = sys.argv[2:6]
text = path.read_text()

# version "X.Y.Z"
text, n = re.subn(r'(^\s*version\s+")[^"]+(")', rf'\g<1>{version}\g<2>', text, count=1, flags=re.M)
assert n == 1, "version line not found"

# Three sha256 lines in order: arm mac, intel mac, linux
def replace_nth(pattern, repls, src):
    out = []
    last = 0
    matches = list(re.finditer(pattern, src, flags=re.M))
    assert len(matches) == len(repls), f"expected {len(repls)} sha256 lines, found {len(matches)}"
    for m, repl in zip(matches, repls):
        out.append(src[last:m.start()])
        out.append(re.sub(r'"[0-9a-f]{64}"', f'"{repl}"', m.group(0)))
        last = m.end()
    out.append(src[last:])
    return ''.join(out)

text = replace_nth(r'^\s*sha256\s+"[0-9a-f]{64}"\s*$', [arm_mac, intel_mac, linux], text)

path.write_text(text)
PY

echo "==> diff"
git -C "${TAP_DIR}" --no-pager diff -- Formula/redmine.rb || true

echo
echo "Done. Next steps (run manually after review):"
echo "  cd ${TAP_DIR}"
echo "  git add Formula/redmine.rb"
echo "  git commit -m 'bump redmine to ${TAG}'"
echo "  git push origin master"
