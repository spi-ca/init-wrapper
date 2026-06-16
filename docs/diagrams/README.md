# Mermaid 다이어그램 안내

`docs/diagrams`는 이 저장소의 **독립 Mermaid 소스 파일**을 모아 두는 디렉터리입니다. 현재는 README의 설명을 보조하는 모듈 아키텍처 다이어그램 하나를 관리합니다.

## 현재 포함된 파일

- `module-architecture.mmd`: `init-wrapper`의 Rust 모듈 책임과 의존 흐름
- `mermaid-config.json`: 저장소 공용 Mermaid 렌더링 스타일

README 안에 들어 있는 간단한 컴포넌트/시퀀스 다이어그램은 `README.md` 자체가 source of truth이며, 이 디렉터리에는 별도 `.mmd`로 복제하지 않습니다.

## 렌더링 규칙

- 수정의 기준 파일은 `*.mmd`입니다.
- 렌더 산출물(`.svg`, `.png`)은 필요할 때만 생성합니다.
- 별도 브라우저 launch 설정 파일은 현재 저장소에 추적하지 않습니다. Mermaid CLI가 요구하는 Chromium/Puppeteer 준비는 실행 환경에서 해결합니다.

## 예시 명령

단일 SVG 렌더링:

```bash
bunx @mermaid-js/mermaid-cli \
  -i docs/diagrams/module-architecture.mmd \
  -o docs/diagrams/module-architecture.svg \
  -c docs/diagrams/mermaid-config.json
```

단일 PNG 렌더링:

```bash
bunx @mermaid-js/mermaid-cli \
  -i docs/diagrams/module-architecture.mmd \
  -o docs/diagrams/module-architecture.png \
  -c docs/diagrams/mermaid-config.json \
  --scale 2
```

## 변경 후 확인

- 다이어그램 설명이 `README.md`와 모순되지 않는지 검토합니다.
- 특히 lower 설명은 현재 구현처럼 `/`를 bind mount한 뒤 readonly remount 하는 흐름과 맞아야 합니다.
- 렌더 산출물을 생성했다면 사람이 읽을 수 있는지 직접 열어 확인합니다.
- 문서 변경 후에는 `git diff --check`로 공백 오류를 확인합니다.
