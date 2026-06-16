---
name: iterative-findings-loop
description: Repeat implementation or documentation updates through subagent QA and review until findings and blockers are cleared with current evidence.
---

# Iterative Findings Loop

이 skill은 구현 작업이나 문서 작업에서 QA/리뷰 findings, blocker가 사라질 때까지 반복 루프를 운영하기 위한 절차다.

## When to Use

- subagent 기반 구현 workflow에서 QA 또는 리뷰 피드백을 반영해 반복 수정해야 할 때
- 코드, 테스트, 문서, prompt, skill 변경이 한 번의 pass로 끝나지 않을 가능성이 높을 때
- 완료 전에 모든 명시 요구사항을 current evidence에 매핑해야 할 때

## Procedure

1. 현재 요구사항과 최신 계획을 기준으로 완료 조건을 다시 적는다.
2. 현재 findings를 blocking / non-blocking / unclear로 분류한다.
3. findings가 나온 역할별 산출물을 분리해 유지한다. 최소한 구현, QA, 리뷰 결과를 섞지 않는다.
4. 수정이 필요한 범위를 다시 work package로 나눈다.
   - 독립 파일 범위면 `software-developer` 병렬 lane으로 재실행한다.
   - 공유 파일이나 통합 이슈면 `software-implementer` 중심의 순차 수정으로 처리한다.
5. 수정 후 focused validation을 바로 실행한다.
6. 그 다음 `software-qa`와 `software-reviewer`를 가능하면 병렬로 다시 실행한다.
7. 새 findings가 나오면 원인과 영향 범위를 갱신하고, 필요한 단계만 다시 반복한다.
8. 모든 blocker가 사라지고 non-blocking 항목도 수용 가능하게 정리될 때까지 반복한다.
9. 종료 직전에 모든 명시 요구사항을 파일, diff, 명령 출력, 테스트, 리뷰 결과 같은 current evidence에 매핑한다.

## Parallel Guidance

- 병렬 실행은 파일 범위와 의사결정 범위가 겹치지 않을 때만 사용한다.
- QA와 리뷰는 가능한 병렬로 돌린다.
- 문서 수정과 코드 수정이 서로 독립이면 lane을 분리할 수 있지만, 같은 파일을 건드리면 병렬화하지 않는다.

## Pitfalls

- findings를 단순 나열만 하고 수정 루프로 연결하지 않는다.
- 실패한 검증 상태에서 완료로 표시하지 않는다.
- 같은 shared file을 여러 lane에 동시에 배정하지 않는다.
- 오래된 QA/review 결과를 current evidence처럼 취급하지 않는다.
- blocker가 남아 있는데 문구만 완화해서 종료하지 않는다.

## Verification

- 최신 QA 결과에 blocking issue가 없다.
- 최신 리뷰 결과에 request changes가 없다.
- 요구사항의 모든 명시 항목이 current evidence에 매핑된다.
- 필요한 코드/문서/테스트 변경이 실제 파일 diff에 반영되어 있다.
