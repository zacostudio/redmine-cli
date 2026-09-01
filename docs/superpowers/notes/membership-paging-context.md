# membership list 페이지네이션 — context notes

## 2026-09-01 발견 경위

"admin 이 아니면 사람을 어떻게 찾나" 를 확인하다가 나왔다. 담당자를 다른 사람에게 넘기려면
받는 사람의 숫자 user id 가 필요한데, 공식 `GET /users.json?name=` 은 admin 전용이라
(redmine.org Rest_Users, "This endpoint requires admin privileges") 일반 계정에서는 403 이 난다.
실제 서버(rsupport)에서 `redmine users --name 장` 을 실행해 `HTTP 403` 을 확인했다.

그래서 비관리자의 실질적인 경로는 `GET /projects/:id/memberships.json` 인데,
여기서 **결과가 25명에서 잘리고 있었다.**

## 2026-09-01 근본 원인

`list_memberships` 가 쿼리 파라미터를 빈 배열(`&[]`)로 보내고 있었다. Redmine 의 기본 페이지
크기가 25라 그 이상은 돌아오지 않고, `membership list` 에는 `--limit` / `--offset` 옵션 자체가
없어서 사용자가 늘릴 방법도 없었다. `issues`, `projects`, `news list` 등 다른 목록 명령에는
모두 `--limit` 이 있었으므로 여기만 빠진 누락이다.

측정값: 프로젝트 2539 (`[RM] 3.x 공용 이슈`) 는 `total_count` 가 59인데 25건만 반환됐다.
`total_count` 는 출력에 이미 들어 있었으므로, 잘렸다는 사실 자체는 보이지만 원인은 보이지 않는
상태였다.

## 2026-09-01 결정

**로컬 캐시는 넣지 않기로 했다.** 처음 나온 대안은 사용자 목록을 config.yml 옆에 저장해 두는
것이었는데, 세 가지 이유로 접었다.

- 속도 문제가 아니다. `membership list` 실측이 0.17초 / 0.06초 / 0.06초다. 캐시로 아낄 시간이 없다.
- 캐시는 이 버그를 못 고친다. 오히려 잘린 25명을 파일에 굳혀서 원인 추적만 어려워진다.
- "전체 사용자 캐시" 는 만들 수가 없다. `/users.json` 이 admin 전용이라 비관리자는 전체 목록을
  받을 수 없고, 프로젝트를 하나씩 긁어 합쳐도 자신이 속하지 않은 프로젝트의 사람은 빠진다.

담당자 별명(`정현경 -> 316`)을 custom field alias 처럼 config.yml 에 저장하는 안도 함께
검토했으나, 페이지네이션이 고쳐지면 실익이 작다고 판단해 보류했다. 하루에 여러 번 재할당하는
사용 패턴이 생기면 그때 다시 꺼낼 것.

**기본값은 25로 두었다.** `projects` / `issues` 와 같은 값이라야 명령마다 다른 기본값을 외울
필요가 없다. 전원을 받으려면 `--limit 100` (Redmine 의 상한) 을 쓴다.

## 2026-09-01 검증

TDD 로 진행했다. `cli_parse.rs` 의 파싱 테스트 2개는 `ListArgs` 에 필드가 없어 컴파일 에러로
실패했고, `integration.rs` 의 wiremock 테스트는 `error: unexpected argument '--limit' found` 로
실패했다 — 둘 다 기능 부재가 원인인 것을 확인한 뒤 구현했다.

구현 후 전체 106개 테스트 통과, clippy `-D warnings` 통과, fmt 통과.
실제 서버 확인: 기본은 25/59, `--limit 100` 은 59/59. `--offset 5` 는 6번째 멤버부터 반환된다.

## 2026-09-01 나머지 목록 API 전수 확인

첫 조사에서 `grep '&\[\],'` 로 훑었더니 뒤에 쉼표가 없는 `list_queries` 를 놓쳤다.
**패턴 grep 은 이 용도로 신뢰할 수 없다.** 대신 두 단계로 가렸다.

1. 응답 타입에 `total_count` 필드가 있는데 client 함수가 파라미터를 빈 배열로 보내는 것을 찾는다.
   Redmine 은 페이지네이션하는 엔드포인트에만 `total_count` 를 준다.
2. 후보를 실제 서버에 `?limit=1` 로 찔러 본다. **1건이 오면 페이지네이션, 전부 오면 아니다.**

결과 (rsupport 실측):

| 엔드포인트 | `limit=1` 응답 | 판정 |
|---|---|---|
| `/queries.json` | 1 / total_count 186 | **페이지네이션 — 버그였음** |
| `/projects/:id/versions.json` | 3 / total_count 3 | 아님 (limit 무시) |
| `/projects/:id/issue_categories.json` | 8 / total_count 8 | 아님 (limit 무시) |
| `/projects/:id/files.json` | 7, total_count 키 없음 | 아님 |
| `/projects/:id/wiki/index.json` | total_count 키 없음 | 아님 |

`versions` 와 `issue_categories` 는 응답에 `total_count` 가 있어서 1단계 후보로 잡혔지만,
`limit=1` 에 각각 3건과 8건을 돌려주므로 limit 을 아예 보지 않는다. **`total_count` 존재만으로는
판정할 수 없고 2단계 확인이 반드시 필요하다.**

`files` 는 프로젝트 2539 가 403 이라 파일이 있는 212 로 확인했다.

## 2026-09-01 query 수정

`redmine query` 가 저장된 쿼리 186개 중 25개만 반환하고 있었다. `membership list` 와 같은 원인,
같은 처방이다. `Command::Query` 가 인자 없는 unit variant 였으므로 `QueryArgs` 를 새로 만들어
붙였다.

실측: 기본 25/186, `--limit 100` 은 100건, `--limit 100 --offset 100` 은 86건 (100+86=186).

## 2026-09-01 코드 리뷰 후 — 판정 기준이 틀렸다

`/code-review` 가 `search_users` 와 `list_time_entries` 를 짚었고, 확인해 보니 맞았다.
**앞 절의 2단계 방법은 `offset` 누락을 구조적으로 못 잡는다.** 1단계 조건이 "파라미터를 빈
배열로 보내는 함수" 였는데, 이 둘은 `name` 이나 필터와 함께 `limit` 은 보내고 있어서 후보에
아예 오르지 못했다. `&[]` grep 이 못 미더워서 바꾼 기준이 같은 종류의 구멍을 남긴 것이다.

올바른 기준은 **응답 타입에 `total_count` 가 있는데 `limit` 또는 `offset` 중 하나라도 안 보내는
것** 이다. CLI 층에서 파라미터를 조립하는 명령(`issues`, `search`, `news`, `time-entry`)은
client 함수만 봐서는 알 수 없으므로 `src/cli/*.rs` 의 `pub limit` / `pub offset` 도 같이 봐야 한다.

이 기준으로 다시 훑은 결과:

| 명령 | limit | offset | 판정 |
|---|---|---|---|
| `issues` | O | O | 정상 |
| `projects` | O | O | 정상 |
| `search` | O | O | 정상 |
| `news list` | O | O | 정상 |
| `membership list` | O | O | 이번에 고침 |
| `query` | O | O | 이번에 고침 |
| `time-entry list` | O | **X** | **고침** |
| `users` | O | **X** | **고침** |

`time_entries.json` 은 실측 total_count 가 51,369 이고 `offset=3` 이 다른 구간을 돌려준다.
`--offset` 이 없으면 5만 건 중 앞의 25건 바깥은 볼 방법이 없었다.

`users.json` 은 admin 전용이라 이 계정으로는 실측이 불가능했다(403). `UsersResponse` 에
`total_count` 가 있고 공식 문서 예시에도 페이지 정보가 있다는 근거로 고쳤다 — **실서버 검증은
못 했다.**

`limit` 상한은 100 으로 실측 확인했다. `limit=200`, `limit=500` 모두 100건만 온다.

## 2026-09-01 기본값 전송 회귀 테스트

리뷰가 "기본값이 실제로 전송되는지 지키는 테스트가 없다" 고 지적했고, 확인해 보니 사실이었다.
`list_memberships` 를 다시 `&[]` 로 되돌려도 테스트가 전부 통과했다. 기존 기본 경로 테스트가
`path()` 만 보고 쿼리 문자열은 보지 않았기 때문이다.

`membership_list_returns_json` 과 `query_list_returns_json` 에 `query_param("limit","25")` /
`query_param("offset","0")` 을 넣었다. 같은 회귀를 다시 심어서 이번에는 FAILED 가 나는 것을
확인했다.

## 남은 것 (판단 필요)

리뷰가 짚은 것 중 손대지 않은 것들. 전부 이 변경 이전부터 있던 사안이다.

- **기본값 25 유지.** `membership list` 를 그냥 치면 여전히 59명 중 25명만 온다. `--all` 이나
  자동 페이징을 넣으면 해결되지만, 다른 8개 명령과 기본값이 달라진다. 일관성을 택했다.
- **`limit` 범위 검증 없음.** `--limit 0` 은 서버가 25로, `--limit 500` 은 100으로 조용히
  바꾼다. `value_parser!(u64).range(1..=100)` 이면 즉시 에러가 된다. 다만 같은 문제가
  `projects`, `news`, `search` 등에도 있어 여기만 고치면 오히려 어긋난다.
- **출력에 실제 적용된 `limit`/`offset` 없음.** `search` 는 돌려주는데(`src/cli/search.rs:62-64`)
  나머지는 `total_count` 만 준다. 서버가 값을 깎았는지 호출자가 알 수 없다.
- **`limit`/`offset` 선언 중복 8곳.** `#[command(flatten)]` 으로 공유 `PageArgs` 를 만들면
  이번 같은 누락이 "flatten 이 없다" 로 눈에 띈다. 구조 변경이라 별도 작업으로 남긴다.
