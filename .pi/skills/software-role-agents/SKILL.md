---
name: software-role-agents
description: Run a software-writing workflow with Pi subagents for user representative, systems engineer, designer, parallel-capable developer, implementer, QA, and reviewer roles.
---

# Software Role Agents

이 skill은 software 작성 작업을 역할 기반 Pi subagent workflow로 진행하기 위한 절차다.

현재 명시 역할은 7개다.

1. 사용자 대표 (`user-representative`)
2. 시스템 엔지니어 (`software-systems-engineer`)
3. 설계자 (`software-designer`)
4. 개발자 (`software-developer`)
5. 구현자 (`software-implementer`)
6. QA (`software-qa`)
7. 리뷰어 (`software-reviewer`)

## When to Use

- 새 기능, 버그 수정, 리팩터링, 문서화 등 software 변경을 체계적으로 진행할 때
- 사용자 요구사항, 시스템 제약, 설계, 구현, 검증, 리뷰를 분리해야 할 때
- early boot, PID 1, mount namespace, OS 권한, dependency, runtime 제약처럼 시스템 엔지니어링 검토가 중요한 작업일 때
- `subagent` 도구로 역할별 독립 context를 사용하고 싶을 때

## Installed pi-subagent Frontmatter Policy

현재 설치된 `pi-subagent`는 subagent markdown frontmatter에서 다음 필드를 사용한다.

- `name`: agent 이름
- `description`: agent 설명
- `model`: Pi 모델 ID. Codex 계열은 `openai-codex/gpt-5.4` 또는 `openai-codex/gpt-5.4-mini`처럼 provider/model 형식으로 쓴다.
- `thinking`: `off`, `minimal`, `low`, `medium`, `high`, `xhigh` 중 하나. 모델 ID suffix로 붙이지 않는다.
- `tools`: comma-separated built-in tool 목록

이 프로젝트의 역할 agents는 Codex 계열 모델과 분리된 `thinking` 필드를 사용한다.

## Role Responsibilities

### 사용자 대표

- 사용자 원문 의도와 성공 기준을 보존한다.
- acceptance criteria, 사용자-facing scenario, blocker를 정리한다.
- 모호함이 완료를 막는지 판단한다.

### 시스템 엔지니어

- OS, runtime, dependency, permission, process lifecycle, deployment, 운영 제약을 검토한다.
- 성능, 자원 사용, concurrency, failure recovery, observability 위험을 식별한다.
- 환경 증거가 부족하면 필요한 명령이나 사용자 입력을 명확히 요구한다.

### 설계자

- 요구사항과 시스템 제약을 구현 가능한 설계로 바꾼다.
- 변경 범위, module boundary, error semantics, 검증 전략을 정의한다.
- 구현자가 실행할 수 있는 단계별 계획을 작성한다.

### 개발자

- 승인된 독립 work package 하나를 구현한다.
- 여러 `software-developer` lane이 병렬 실행될 수 있도록 허용 파일, 책임, 검증 범위를 지킨다.
- 공유 파일 충돌이나 package 경계 밖 결정이 필요하면 추측하지 않고 blocker로 보고한다.
- 역할에 맞게 `openai-codex/gpt-5.4`, `thinking: high`, edit-capable tools를 사용해 구현과 focused validation을 수행한다.

### 구현자

- 승인된 계획에 따라 파일을 수정한다.
- 병렬 개발 lane 결과가 있으면 공유 파일 통합과 마무리 구현을 수행한다.
- 기존 사용자 변경과 기존 동작을 보존한다.
- 미승인 shortcut, 임시 미완성 표식, dead code, 중복 로직, 숨은 compatibility shim을 남기지 않는다.
- 관련 검증을 실행하고 결과를 보고한다.

### QA

- acceptance criteria를 command, test, smoke check, artifact inspection에 매핑한다.
- happy path, edge case, failure path를 검증한다.
- 실패를 재현 가능하게 기록하고 blocking 여부를 구분한다.

### 리뷰어

- diff, 파일 내용, QA 증거, 요구사항 매핑을 검토한다.
- correctness, maintainability, security, performance, docs consistency를 점검한다.
- completion audit을 수행하고 approve/request changes를 판단한다.

## Role Input/Output Contract

| 역할 | 주요 입력 | 주요 출력 |
| --- | --- | --- |
| 사용자 대표 | 사용자 원문 요청, 기존 문서, 현재 동작 증거 | 사용자 의도, acceptance criteria, 사용자-facing scenario, 질문/blocker |
| 시스템 엔지니어 | 사용자 대표 산출물, 저장소/환경 정보, dependency/build/runtime 정보 | 시스템 제약, feasibility, 운영 리스크, 시스템 수준 검증 |
| 설계자 | 사용자 대표 산출물, 시스템 엔지니어 산출물, 코드/문서 구조 | 설계 결정, 변경 범위, 구현 계획, 검증 전략 |
| 개발자 | 설계자 계획의 독립 work package, 허용 파일 범위, acceptance criteria, 프로젝트 지침 | package별 파일 변경, focused validation 결과, 병렬 안전성/충돌 보고, blocker |
| 구현자 | 설계자 계획, 개발자 lane 결과, 현재 파일 내용, 프로젝트 지침 | 파일 변경, 공유 파일 통합, 변경 요약, focused validation 결과, blocker |
| QA | acceptance criteria, 구현 diff, 검증 명령/환경 | QA matrix, command/test/smoke evidence, blocking/non-blocking findings |
| 리뷰어 | 사용자 요구사항, 설계, diff, QA 결과 | 품질 리뷰, blocking issue, completion audit, approve/request changes verdict |

## Recommended Subagent Chain

작업이 충분히 크면 다음 chain을 사용한다. `parallel-development` 단계는 설계자가 non-overlapping work package를 분리했을 때만 사용한다. package가 하나뿐이면 단일 `software-developer` lane으로 실행하고, 안전하게 분리할 수 없으면 해당 단계를 건너뛰고 `software-implementer`가 순차 구현한다.

```json
{
  "chain": [
    {
      "label": "requirements",
      "agent": "user-representative",
      "task": "Summarize user intent, acceptance criteria, user-facing scenarios, constraints, and blockers for: <task>"
    },
    {
      "label": "system-constraints",
      "agent": "software-systems-engineer",
      "task": "Using the requirements output, inspect repository/environment constraints and provide system-level feasibility, risks, and verification for: <task>"
    },
    {
      "label": "design",
      "agent": "software-designer",
      "task": "Using requirements and system constraints, design the implementation plan and verification strategy for: <task>"
    },
    {
      "type": "parallel",
      "label": "parallel-development",
      "tasks": [
        {
          "agent": "software-developer",
          "task": "Implement independent work package A for: <task>. Include allowed files, acceptance criteria, preserved behavior, and focused validation."
        },
        {
          "agent": "software-developer",
          "task": "Implement independent work package B for: <task>. Include allowed files, acceptance criteria, preserved behavior, and focused validation."
        }
      ]
    },
    {
      "label": "implementation-merge",
      "agent": "software-implementer",
      "task": "Integrate developer lane results, resolve approved shared-file work, and run focused validation for: <task>"
    },
    {
      "type": "parallel",
      "label": "verification-review",
      "tasks": [
        {
          "agent": "software-qa",
          "task": "Validate the implementation against acceptance criteria for: <task>"
        },
        {
          "agent": "software-reviewer",
          "task": "Review the implementation, evidence, maintainability, and completion readiness for: <task>"
        }
      ]
    }
  ],
  "mode": "spawn"
}
```

## Single-Agent Fallback

`subagent` 사용이 어렵거나 작업이 작으면 root agent가 같은 순서를 role-play로 수행한다.

1. 사용자 대표: 의도와 acceptance criteria 정리
2. 시스템 엔지니어: 환경/운영 제약 확인
3. 설계자: 계획 작성
4. 개발자: 독립 package 구현 또는 병렬 lane 역할 수행
5. 구현자: 변경 적용/통합
6. QA: 검증 실행
7. 리뷰어: completion audit

## Quality Gates

완료 전 반드시 확인한다.

- 모든 적용 대상 역할의 산출물이 존재하거나, 작은 작업이면 root agent가 각 적용 대상 역할 관점의 결과를 보고했다.
- `software-developer`는 독립 work package가 있을 때 적용 대상이다. 사용했다면 각 lane의 allowed files, changed files, validation, conflict 여부가 보고되었고, 사용하지 않았다면 안전하게 분리할 package가 없다는 근거가 보고되었다.
- 사용자 요구사항의 모든 명시 항목이 파일 내용, command output, diff, test, log 같은 current evidence에 매핑되었다.
- 시스템 제약과 운영 위험이 검토되었다.
- 기존 동작과 사용자 변경을 보존했다.
- 변경 유형에 맞는 저장소 검증 명령을 실행했다. 문서/Pi resource 전용 변경은 관련 문서 존재 확인, targeted grep, `git diff --check` 같은 focused validation을 따르고, Rust 코드 변경이 포함되면 fmt/clippy/test 같은 코드 검증을 추가한다.
- 미승인 shortcut, 임시 미완성 표식, dead code, duplicated logic, 숨은 가정, 문서화되지 않은 behavior change가 없다.
- QA와 리뷰어 결과가 blocking issue 없이 완료 가능하다고 판단했다.

## Output Template

```markdown
## 역할별 결과

### 사용자 대표
- 목표:
- Acceptance criteria:
- Blocker:

### 시스템 엔지니어
- 시스템 제약:
- 운영 리스크:
- 시스템 검증:

### 설계자
- 설계 결정:
- 변경 대상:
- 검증 전략:

### 개발자
- Work packages:
- 병렬 안전성:
- 변경 파일:

### 구현자
- 변경 파일:
- 주요 변경:
- 통합 결과:

### QA
- 실행 명령:
- 결과:

### 리뷰어
- Audit:
- 남은 이슈:
- 완료 여부:
```

## Pitfalls

- 명시된 역할을 수 맞추기 때문에 누락하지 않는다.
- 시스템 엔지니어와 설계자를 같은 역할로 합치지 않는다. 시스템 엔지니어는 환경/운영 제약을 제공하고 설계자는 구현 구조를 결정한다.
- QA와 리뷰어를 같은 책임으로 합치지 않는다. 병렬 단계로 묶을 수는 있지만 산출물은 분리한다.
- 계획만 만들고 구현/검증이 남은 상태에서 완료하지 않는다.
- 실패한 검증을 단순 보고로 끝내지 말고 가능한 경우 원인을 수정하고 재검증한다.
