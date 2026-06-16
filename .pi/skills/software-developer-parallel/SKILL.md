---
name: software-developer-parallel
description: Split approved software work into independent packages and run parallel software-developer subagents safely.
---

# Software Developer Parallel

이 skill은 승인된 설계나 명확한 software 변경 요청을 독립 work package로 나누고, `software-developer` subagent 여러 개를 병렬 실행하기 위한 절차다.

## When to Use

- 구현 범위가 여러 독립 파일, 모듈, 문서, 테스트로 나뉘어 병렬 처리 이점이 있을 때
- 각 lane의 변경 범위와 검증 기준을 명확히 분리할 수 있을 때
- 기존 `software-role-agents` workflow의 design 이후 implementation 단계를 여러 개발자 lane으로 확장하고 싶을 때

사용하지 않는다:

- 변경 파일이 대부분 동일해 lane 충돌 가능성이 높을 때
- 요구사항이나 설계가 아직 불명확해 병렬 구현보다 의사결정이 먼저 필요할 때
- 한 lane의 결과가 다른 lane의 입력으로 필요한 순차 작업일 때

## Agent Configuration

`software-developer`는 다음 설정을 사용한다.

- Model: `openai-codex/gpt-5.4`
- Thinking: `high`
- Tools: `read`, `grep`, `find`, `ls`, `bash`, `edit`, `write`

이 설정은 병렬 lane에서도 각 개발자가 파일 내용, 저장소 지침, 검증 결과를 충분히 추론하고 안전하게 편집해야 하므로 high reasoning과 edit-capable tool을 사용한다. 단순 QA나 요구사항 정리보다 구현 충돌·검증 실패 triage 비용이 높아 mini 모델보다 full Codex 모델을 기본값으로 둔다.

## Procedure

1. 현재 요청, 승인된 설계, 저장소 지침, 기존 사용자 변경사항을 확인한다.
2. work package 후보를 파일/모듈/책임 단위로 나눈다.
3. 각 package에 다음을 명시한다.
   - 목표와 acceptance criteria
   - 허용 변경 파일 또는 디렉터리
   - 보존해야 할 동작
   - 실행할 focused validation
   - 다른 lane과 공유하면 안 되는 파일
4. package 간 파일 범위가 겹치면 병렬 실행하지 말고 순차 작업이나 root merge step으로 바꾼다.
5. 독립 package만 `subagent` parallel mode로 실행한다.
6. lane 결과를 모아 diff, validation, blocker를 확인한다.
7. 공유 파일 통합, 문서 정합성, 최종 검증은 root agent나 별도 순차 step에서 수행한다.
8. `software-qa`와 `software-reviewer`로 검증·리뷰를 수행한다.
9. findings 또는 blocker가 있으면 package 재분할 또는 수정 계획을 갱신하고 필요한 lane만 다시 실행한다.
10. 모든 requirement가 current evidence에 매핑되고 blocker가 없을 때만 완료한다.

## Parallel Subagent Template

```json
{
  "tasks": [
    {
      "agent": "software-developer",
      "task": "Implement work package A for <task>. Allowed files: <paths>. Acceptance criteria: <criteria>. Preserve existing behavior outside this package. Run focused validation: <commands>. Report blockers and parallel-safety conflicts."
    },
    {
      "agent": "software-developer",
      "task": "Implement work package B for <task>. Allowed files: <paths>. Acceptance criteria: <criteria>. Preserve existing behavior outside this package. Run focused validation: <commands>. Report blockers and parallel-safety conflicts."
    }
  ],
  "mode": "spawn"
}
```

## Integration with Role Workflow

기존 역할 workflow에서는 다음처럼 사용한다.

1. `user-representative`: 목표와 acceptance criteria 정리
2. `software-systems-engineer`: 시스템 제약 검토
3. `software-designer`: work package로 나눌 수 있는 구현 계획 작성
4. `software-developer` lanes: 독립 package 병렬 구현
5. root merge step 또는 `software-implementer`: 공유 파일 통합과 focused validation
6. `software-qa`와 `software-reviewer`: 병렬 검증과 리뷰

## Quality Gates

- 각 lane의 allowed files와 실제 changed files가 일치한다.
- lane 간 같은 파일을 동시에 수정하지 않았다.
- 모든 변경은 승인된 package 목표와 연결된다.
- focused validation과 repository-level validation이 fresh command output으로 남아 있다.
- QA/review 결과에 blocking findings가 없다.
- 미승인 shortcut, 임시 미완성 표식, dead code, duplicated logic, compatibility shim, 숨은 가정, 문서화되지 않은 behavior change가 없다.

## Output Template

```markdown
## Parallel Developer Plan

| Package | Agent | Allowed files | Acceptance criteria | Validation |
| --- | --- | --- | --- | --- |
| ... | `software-developer` | ... | ... | ... |

## Lane Results
- Package:
  - Files changed:
  - Validation:
  - Blockers/conflicts:

## Merge and Review
- Integrated files:
- QA evidence:
- Review verdict:
- Remaining findings/blockers:
```

## Pitfalls

- 병렬 처리를 위해 불명확한 scope를 억지로 나누지 않는다.
- 같은 파일을 여러 lane에 배정하지 않는다. 공유 파일 변경은 root merge step으로 남긴다.
- 한 lane의 미검증 결과를 다른 lane의 성공으로 간주하지 않는다.
- project-local subagent 실행 확인을 우회하지 않는다.
- validation 실패를 단순 보고로 끝내지 말고 가능한 경우 수정 후 재검증한다.
