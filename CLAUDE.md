# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

**응답 언어:** 사용자와의 모든 응답·커뮤니케이션은 **한국어**로 한다. 코드·식별자·로그·명령어 등 기술 토큰과 인용 원문은 그대로 둔다.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## 5. End-of-Task Wrap-up

**코드 변경 작업 단위가 끝나면 문서 업데이트 → 로컬 commit까지 한 번에 마무리한다. 주요 작업 단위(계획의 Task 하나가 검토를 통과한 시점)가 끝나면 확인 없이 push 한다. force push는 사용자 확인 후에만 한다.**

다음 조건 중 하나가 만족되면 작업 단위가 끝난 것으로 본다:
- 한 묶음의 기능/수정이 빌드·테스트 검증까지 끝난 시점
- 사용자가 "끝", "다음", "정리해줘" 같은 전환 신호를 준 시점
- Stop hook이 uncommitted 변경을 system message로 알린 시점

수행 절차:
1. 변경 내용을 한두 줄로 요약해 [README.md](README.md) §"현재 반영 상태"와 [docs/PROJECT_GUIDE.md](docs/PROJECT_GUIDE.md) §"현재 참고 상태"에 추가한다. 중간 시행착오나 디버깅 잡일은 git history에 맡기고 문서엔 사용자 가시 효과만 적는다.
2. `git status`로 변경 파일을 확인한다. 의도치 않은 파일이 섞여 있으면 사용자에게 알린다.
3. 작업 단위가 끝나면 로컬 commit은 확인 없이 진행한다.
4. **주요 작업 단위(계획의 Task 하나가 검토를 통과한 시점)가 끝나면 확인 없이 push 한다.** force push는 사용자 확인 후에만 한다.
5. **commit 메시지 끝에 반드시 `문서 갱신: README §현재 반영 상태 + PROJECT_GUIDE §현재 참고 상태` 한 줄을 붙인다.** 이 줄을 적을 수 없으면(=아직 안 갱신함) commit하지 말고 1번으로 돌아가 먼저 갱신한다. 문서 변경 없이 코드만 staged면 누락 신호로 보고 멈춘다.

이 룰의 목적: 진척 문서와 코드 변경의 정합성 유지, 그리고 검토되지 않은 변경이 GitHub에 올라가는 것 방지.

## 6. 템플릿 우선 UI

**이 프로젝트는 `SDTPL_ADM`(shadcn UI Kit 클론) 템플릿 기반이다. 새 UI는 자작하기 전에 반드시 템플릿부터 뒤진다.**

새 페이지·컴포넌트를 만들기 전에:
- 먼저 `web/src/components/ui/`(버튼·카드·테이블 등 프리미티브), `web/src/components/dashboards/`·`web/src/components/pages/`·`web/src/components/apps/`(완성 패턴)를 검색해 재사용 가능한 것을 찾는다.
- 비슷한 화면이 이미 있으면 그 구조·레이아웃을 그대로 따른다. 처음부터 새로 짜지 않는다.
- UI를 새로 짜기 전에 원본 `c:\Wspace\real-es\src` 에 같은 화면이 있는지 먼저 본다.
- 프리미티브를 직접 만들지 않는다. 없으면 멈추고 사용자에게 알린다.

자작 UI가 정말 필요할 때만:
- 타이포·간격·색·radius는 추측하지 말고 기존 컴포넌트와 토큰(Tailwind 클래스·CSS 변수)을 그대로 따른다. 임의 값(`px`·hex) 하드코딩 금지.
- 기존 패턴에서 벗어나는 이유를 한 줄로 설명할 수 있어야 한다.

목적: 템플릿의 일관된 디자인 언어 유지, 어긋나는 자작 UI로 인한 재작업 방지.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.
