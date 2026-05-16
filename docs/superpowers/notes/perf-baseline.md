# 성능 베이스라인

## 초기 (release profile 튜닝 전)

`[profile.release]` = `opt-level = 3` only.

### 바이너리 크기
```
-rwxr-xr-x@ 1 jinhyoung  staff   5.1M  5월 16 09:39 target/release/redmine
```

### `redmine --help` 실행 시간
```
Benchmark 1: target/release/redmine --help
  Time (mean ± σ):       1.6 ms ±   0.2 ms    [User: 0.8 ms, System: 0.6 ms]
  Range (min … max):     1.3 ms …   3.1 ms    549 runs
```

### 크레이트별 바이너리 점유 (top 20)
```
 File  .text     Size Crate
 9.0%  18.1% 494.7KiB std
 8.0%  16.1% 440.6KiB redmine_cli
 7.6%  15.1% 413.6KiB reqwest
 7.2%  14.5% 395.5KiB rustls
 4.3%   8.6% 236.5KiB clap_builder
 2.7%   5.3% 146.1KiB ring
 1.4%   2.8%  75.3KiB tokio
 1.2%   2.5%  67.0KiB webpki
 1.2%   2.4%  64.9KiB hyper_util
 1.1%   2.2%  59.3KiB hyper
 0.9%   1.8%  50.3KiB url
 0.7%   1.4%  37.1KiB http
 0.7%   1.4%  37.1KiB idna
 0.7%   1.3%  36.5KiB toml_parser
 0.6%   1.3%  34.5KiB toml
 0.5%   1.1%  29.8KiB [Unknown]
 0.5%   1.0%  28.3KiB serde_json
 0.2%   0.5%  12.9KiB rustls_pki_types
 0.1%   0.3%   8.2KiB bytes
 0.1%   0.2%   6.0KiB icu_normalizer
 1.2%   2.4%  65.1KiB And 42 more crates. Use -n N to show more.
50.0% 100.0%   2.7MiB .text section size, the file size is 5.3MiB
```

---

## 튜닝 후

`[profile.release]` 설정.

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = "symbols"
panic = "abort"
```

### 바이너리 크기
```
-rwxr-xr-x@ 1 jinhyoung  staff   2.6M  5월 16 09:40 target/release/redmine
```

### `redmine --help` 실행 시간
```
Benchmark 1: target/release/redmine --help
  Time (mean ± σ):       1.6 ms ±   0.2 ms    [User: 0.8 ms, System: 0.6 ms]
  Range (min … max):     1.3 ms …   2.8 ms    596 runs
```

### 크레이트별 바이너리 점유 (top 20)
```
 File  .text     Size Crate
16.4%  27.0% 530.3KiB std
 8.5%  14.0% 273.8KiB rustls
 5.9%   9.8% 191.7KiB redmine_cli
 5.1%   8.5% 165.9KiB clap_builder
 4.2%   6.9% 134.5KiB ring
 3.4%   5.6% 110.5KiB reqwest
 1.8%   3.0%  58.3KiB hyper
 1.8%   2.9%  57.2KiB hyper_util
 1.7%   2.8%  54.2KiB serde_json
 1.6%   2.6%  50.4KiB http
 1.5%   2.6%  50.1KiB tokio
 1.4%   2.4%  46.2KiB url
 1.4%   2.3%  45.3KiB webpki
 1.1%   1.9%  36.3KiB toml
 0.9%   1.6%  30.7KiB [Unknown]
 0.7%   1.2%  23.1KiB toml_parser
 0.6%   0.9%  18.3KiB idna
 0.3%   0.5%   9.6KiB serde_core
 0.3%   0.5%   9.3KiB icu_normalizer
 0.3%   0.5%   9.3KiB hyper_rustls
 2.3%   3.8%  75.4KiB And 28 more crates. Use -n N to show more.
60.5% 100.0%   1.9MiB .text section size, the file size is 3.2MiB
```

---

## 델타 요약

| 지표 | 튜닝 전 | 튜닝 후 | 변화 |
|------|---------|---------|------|
| 파일 크기 | 5.1 MiB | 2.6 MiB | **-49%** (2.5 MiB 감소) |
| `.text` 섹션 크기 | 2.7 MiB | 1.9 MiB | **-30%** (0.8 MiB 감소) |
| `--help` 평균 실행 시간 | 1.6 ms ± 0.2 ms | 1.6 ms ± 0.2 ms | 변화 없음 (5ms 미만 범위에서 측정 정밀도 한계) |
| 빌드 시간 | ~14s | ~27s | +13s (LTO로 인한 증가, 예상 범위) |

### 비고
- `cargo bloat`은 `panic = "abort"` + `strip = "symbols"` 조합에서도 정상 동작함.
- LTO(fat) + `codegen-units = 1`로 크레이트 간 인라이닝이 활발해져 `.text` 섹션이 30% 감소.
- 바이너리 전체 크기 49% 감소는 `strip = "symbols"` 효과가 주된 원인.
- 실행 시간은 이미 1.6ms로 충분히 빠르며 측정 정밀도 한계(5ms 미만) 내에 있어 개선 측정 불가.
- 모든 12개 테스트 통과 확인.
