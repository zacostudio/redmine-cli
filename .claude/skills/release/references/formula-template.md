# homebrew-redmine Formula 템플릿

`zacostudio/homebrew-redmine` 저장소의 `Formula/redmine.rb` 구조. 버전 갱신 시 **딱 4곳**만 바뀐다.

```ruby
class Redmine < Formula
  desc "Standalone CLI for Redmine"
  homepage "https://github.com/zacostudio/redmine-cli"
  version "X.Y.Z"                                       # ← (1) 버전
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "<ARM_MAC_SHA256>"                          # ← (2) arm64 mac sha256
    end
    on_intel do
      url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "<INTEL_MAC_SHA256>"                        # ← (3) x86_64 mac sha256
    end
  end

  on_linux do
    url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "<LINUX_SHA256>"                              # ← (4) x86_64 linux sha256
  end

  def install
    bin.install "redmine"
  end

  test do
    assert_match "redmine", shell_output("#{bin}/redmine --version")
  end
end
```

## 주의 사항

- `url` 라인은 절대 바꾸지 말 것. `#{version}` 보간 덕분에 (1)만 갱신하면 URL이 따라간다.
- sha256 값은 64자 소문자 hex. `.sha256` 파일은 `<hex>  <filename>` 형식이므로 첫 토큰만 사용.
- 순서: **arm mac → intel mac → linux**. update-formula.sh도 이 순서를 가정한다.
- `class Redmine < Formula` 클래스명은 파일명(`redmine.rb`)과 매칭되어야 한다. 변경하지 말 것.

## 검증

Formula 파일 갱신 후 로컬에서:

```bash
brew install --build-from-source ./Formula/redmine.rb  # 다운로드 + sha256 검증
brew audit --strict --new-formula ./Formula/redmine.rb # 스타일 점검 (선택)
```

sha256 불일치면 brew가 즉시 실패하므로 가장 흔한 오타가 빠르게 잡힌다.
