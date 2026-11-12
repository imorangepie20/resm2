# RESM 프로젝트 가이드

개발자와 AI 코딩 세션이 이 저장소에서 작업할 때 먼저 읽는 문서다. 작업 규칙은 [CLAUDE.md](../CLAUDE.md).

## 개요

중개사무소용 부동산 업무 도구(매물 관리, 계약 진행, 고객, 일정, 실거래가, 네이버 매물 수집, 엑셀 입출력). 원본 `c:\Wspace\real-es`(Next.js + Prisma + PostgreSQL, 읽기 전용 참고)를 Rust 서버 + React SPA + Tauri 2 앱으로 재작성한다.

## 구조

- `server/` axum API 서버 (`/api/v1`, JSON). 기존 `real_es` DB를 sqlx로 쓴다. 스키마 주인은 원본 `real-es/prisma/` 이며 이 서버는 마이그레이션을 실행하지 않는다.
- `web/` React SPA. 원본의 shadcn UI 컴포넌트를 복사해 쓰고 데이터는 `/api/v1` 로 받는다.
- `app/` Tauri 2 셸. 같은 화면 묶음을 Windows·Android 앱에 담는다.
- `docs/superpowers/specs/`, `docs/superpowers/plans/` 단계별 설계·계획 문서.

## 로드맵

| 단계 | 내용 | 비고 |
|---|---|---|
| 1 | 뼈대: Rust 서버 + 로그인 + 매물 관리, 세 플랫폼에서 동작 | 설계: `docs/superpowers/specs/2026-11-12-resm-rust-stage-1-design.md` |
| 2 | 계약 워크플로·양식 인쇄, 고객(당사자 포함), 일정 | |
| 3 | 엑셀 입출력, 실거래가(통계·지도) | 카카오 지도 키의 앱 출처 등록 확인 필요 |
| 4 | 네이버 매물 수집 | 가장 위험. 착수 전에 Rust에서 봇 차단 우회가 되는지 작은 실험부터 |
| 5 | 설정·회원·알림 화면·홈 대시보드, 이후 도메인 전환 | |

각 단계는 따로 설계 문서를 쓰고 승인받는다.

## 현재 참고 상태

- 2026-11-12: 저장소 뼈대와 1단계 설계·계획 문서만 있다. 다음은 계획서의 Task 1부터.
