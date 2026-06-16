# AGENTS.md

이 저장소에서 작업하는 에이전트는 아래 공통 규칙을 따른다.

## 프로젝트 목적

`init-wrapper`는 Linux 커널의 `init=`로 early boot 시점에 PID 1로 실행되는 작은 Rust 래퍼다. 기존 `/`를 overlayfs lower로 bind mount한 뒤 read-only remount하고, `/run` tmpfs 아래의 upper/work를 사용해 휘발성 overlay root로 `pivot_root`한 다음 실제 init 후보에 `execv`로 제어를 넘긴다.

## 필수 가드레일

- 일반 사용자 공간에서 임의 실행하지 않는다. 이 프로그램은 mount namespace와 root mount를 바꾸는 privileged early-boot 도구다.
- 코드 설명의 source of truth는 `src/main.rs`의 syscall 순서와 `src/unix.rs` wrapper다.
- overlay lower는 bind mount 후 `MS_BIND | MS_REMOUNT | MS_RDONLY` 단계로 read-only화한다는 계약을 유지한다.
- upper/work가 `/run` tmpfs 아래에 있으므로 런타임 쓰기는 휘발성이며 메모리 압박 위험이 있음을 문서화한다.
- 실제 init 후보 순서는 `/sbin/init`, `/usr/sbin/init`, `/usr/lib/systemd/systemd`다.
- `/oldroot/run` 이동은 필수 경로로 취급하되 기존 `EINVAL` 허용 동작을 보존하고, `/oldroot/dev` 이동은 early-boot 환경 차이를 고려해 best-effort로 취급한다.
- root/pivot/boot smoke 검증은 disposable VM 또는 테스트 initramfs에서만 수행한다. 운영 중인 시스템에서 실험하지 않는다.
- 사용자의 기존 변경사항을 덮어쓰지 말고 변경 전후 diff를 확인한다.

## 문서 포인터

- 사용자/운영 개요: [`README.md`](README.md)
- 다이어그램 소스와 렌더링 규칙: [`docs/diagrams/README.md`](docs/diagrams/README.md)
- 에이전트 문서 작성 참고: [`docs/guidelines/`](docs/guidelines/)

## 검증 포인터

문서 또는 `.pi` 변경 시 최소 확인:

```bash
git diff --check
find README.md AGENTS.md docs .pi -maxdepth 3 -type f -print | sort
```

Rust 코드 변경이 포함되면 가능한 범위에서 추가 확인:

```bash
cargo fmt --check
cargo test --all-targets --all-features
```

현재 환경에서 crates.io 접근이나 root 권한이 없으면 실패를 숨기지 말고 정확한 blocker를 기록한다. mount/pivot_root 동작은 disposable boot 환경에서 별도 smoke transcript로 검증해야 한다.
