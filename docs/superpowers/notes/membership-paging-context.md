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

## 남은 것

같은 형태로 쿼리 파라미터를 빈 배열로 보내는 목록 함수가 둘 더 있다 —
`list_files` (`src/client.rs:520`) 와 `list_wiki_pages` (`src/client.rs:542`).
다만 이 두 엔드포인트가 Redmine 쪽에서 실제로 페이지네이션되는지는 **확인하지 않았다.**
(wiki index 는 전체를 반환한다고 알려져 있으나 검증 전이다.) 같은 증상이 있는지 보려면
`total_count` 와 반환 건수를 비교해 보면 된다.
