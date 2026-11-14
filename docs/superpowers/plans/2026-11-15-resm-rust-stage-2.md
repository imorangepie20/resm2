# RESM 2단계(계약·고객·일정) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 계약 워크플로·양식 인쇄, 고객·당사자 관리, 일정 캘린더가 Windows 앱·Android 앱·브라우저 세 곳에서 동작하게 한다. 기존 PostgreSQL `real_es` 스키마 변경 없이 API·화면만 추가한다.

**Architecture:** `server/`(axum + sqlx)에 계약·양식·고객·당사자·일정 API 추가. `web/`(Vite + React)에 계약/인쇄/고객/일정 라우트와 컴포넌트 추가. `app/`(Tauri 2)은 같은 SPA 묶음을 그대로 사용. 공통 로직(`contract-checklist`, `contract-forms`, `calendar/*`, `customers/types`)은 `web/src/lib/`에서 서버·화면이 함께 쓴다(서버는 Rust 포팅).

**Tech Stack:** 1단계와 동일. 추가: 서버 사이드 HTML 템플릿 엔진(`tera` 또는 `askama` 권장).

**Spec:** `docs/superpowers/specs/2026-11-15-resm-rust-stage-2-design.md` — 실행자는 계획과 함께 반드시 읽는다.

## Global Constraints

- 원본 `c:\Wspace\real-es` 와 `c:\Wspace\real-es-desktop` 은 **읽기만** 한다. 파일을 고치지 않는다.
- 새 서버는 DB 마이그레이션을 실행하지 않는다. `CREATE`/`ALTER`/`DROP` 금지. 스키마의 주인은 원본 `prisma/`.
- 테이블·컬럼 이름은 따옴표로 감싼다(`"Property"`, `"userId"`, `"contractChecklist"`). 시각 컬럼은 `TIMESTAMP(3)`(timezone 없음, UTC) → Rust `chrono::NaiveDateTime`.
- 기본키는 서버가 `cuid2::create_id()` 로 만든다. `updatedAt` 은 UPDATE마다 서버가 `now() at time zone 'utc'` 로 넣는다.
- sqlx는 컴파일 시점 매크로(`query!`)를 쓰지 않는다. `sqlx::query` / `query_as` + `FromRow`.
- 세션: 무작위 32바이트 base64url(패딩 없음) 토큰, DB에는 SHA-256 소문자 16진, 만료 30일. bcrypt cost 12. 쿠키 이름 `session`(httpOnly, SameSite=Lax, path `/`).
- 모든 쿼리는 `"userId" = 현재 사용자` 조건. 남의 데이터는 404.
- API 금액은 **원 단위 십진 문자열**. 만원 환산은 `web/` 에서만.
- 날짜 필드는 `YYYYMMDD` 문자열.
- 오류 응답은 `{ "error": "<한국어 메시지>" }`. 가입·로그인·계약·고객·일정 오류 문구는 원본 서버 액션과 글자까지 같게.
- UI는 원본 컴포넌트를 복사해 쓴다. 프리미티브 자작 금지, 임의 `px`·hex 금지, 네이티브 `confirm`/`alert` 금지(`ConfirmDialog`).
- 사용자와의 응답은 한국어. `git push` 는 사용자가 명시적으로 지시할 때만.
- 서버 포트 `3104`. 운영 주소 `https://resm2.approid.team`. Tauri 식별자 `team.approid.resm`.
- Zorin 배포·Cloudflare 설정은 실행 전에 사용자 확인을 받는다.

## Review Focus

각 줄의 테스트는 해당 Task에 들어 있다.

1. 계약 체크리스트 토글 시 낙관적 업데이트 후 서버 실패하면 롤백. 빈 화면·무한 로딩 금지. (Task 6)
2. 계약 완료 처리 시 필수 필드·체크 모두 충족돼야 버튼 활성. 미충족 시 400 `"필수 항목·데이터가 모두 충족되지 않았습니다"`. (Task 1, Task 6)
3. 양식 인쇄 HTML이 원본 `contract-document.tsx` 와 시각적으로 동일(A4, serif, mm 단위, 테이블 border). 서버 템플릿으로 렌더된 HTML이 브라우저 인쇄 대화상자에서 정상 표시. (Task 2, Task 7)
4. 당사자 추가 시 이름+전화(숫자만)로 기존 고객 매칭 → 재사용. 없으면 신규 생성 후 `PropertyParty` 연결. 동일 인물 중복 생성 금지. (Task 4, Task 9)
5. 일정 월 이동·필터 변경 시 로딩 상태 표시. 파생 일정(매물 날짜 필드)이 수기 일정과 함께 정상 표시. 매물 파생 일정 클릭 → 해당 매물 수정 폼 이동. (Task 5, Task 10)
6. 앱에서 서버에 닿지 않는 경우(네트워크 끊김·서버 중지) → 한국어 오류 토스트와 재시도 가능 상태. 흰 화면 금지. (1단계 Task 6과 동일, 새 화면에도 적용)

## File Structure (추가/변경분)

```
resm/
├── server/
│   ├── src/
│   │   ├── contract/          # NEW
│   │   │   ├── mod.rs
│   │   │   ├── model.rs       # ContractData, ChecklistItem, Progress, 요청/응답 DTO
│   │   │   ├── repo.rs        # SQL 쿼리
│   │   │   ├── handlers.rs    # GET /contract/{id}, PATCH checklist, POST start/complete
│   │   │   └── validate.rs    # contractProgress, requiredFieldKeys 포팅
│   │   ├── forms/             # NEW
│   │   │   ├── mod.rs
│   │   │   ├── templates/     # tera/askama 템플릿 6종 + 파셜
│   │   │   ├── engine.rs      # 템플릿 엔진 초기화
│   │   │   └── handlers.rs    # GET /forms/{propertyId}, GET /forms/{propertyId}/{formId}
│   │   ├── customers/         # NEW
│   │   │   ├── mod.rs
│   │   │   ├── model.rs       # CustomerRow, CustomerInput, PartyRow
│   │   │   ├── repo.rs
│   │   │   └── handlers.rs    # GET/POST/PATCH/DELETE /customers, GET /customers/{id}
│   │   ├── parties/           # NEW
│   │   │   ├── mod.rs
│   │   │   ├── model.rs
│   │   │   ├── repo.rs
│   │   │   └── handlers.rs    # GET/POST/DELETE /properties/{propertyId}/parties
│   │   ├── events/            # NEW
│   │   │   ├── mod.rs
│   │   │   ├── model.rs       # EventRow, EventInput, PropertyDateEvent, CalendarOption
│   │   │   ├── repo.rs
│   │   │   └── handlers.rs    # GET/POST/PATCH/DELETE /events
│   │   ├── lib.rs             # MOD: 라우트 등록, 템플릿 엔진 state 추가
│   │   └── tests/
│   │       ├── contract.rs    # NEW
│   │       ├── forms.rs       # NEW
│   │       ├── customers.rs   # NEW
│   │       ├── parties.rs     # NEW
│   │       └── events.rs      # NEW
├── web/
│   ├── src/
│   │   ├── lib/
│   │   │   ├── properties/
│   │   │   │   ├── contract-checklist.ts     # COPY from 원본
│   │   │   │   ├── contract-forms.ts         # COPY from 원본
│   │   │   │   ├── party-role.ts             # COPY from 원본
│   │   │   │   ├── format.ts                 # COPY + formatYmd 추가
│   │   │   │   └── *.test.ts                 # COPY 단위 테스트
│   │   │   ├── calendar/
│   │   │   │   ├── month-grid.ts             # COPY from 원본
│   │   │   │   ├── categories.ts             # COPY from 원본
│   │   │   │   ├── property-events.ts        # COPY from 원본
│   │   │   │   └── *.test.ts                 # COPY 단위 테스트
│   │   │   ├── customers/
│   │   │   │   ├── types.ts                  # COPY from 원본
│   │   │   │   └── types.test.ts             # COPY 단위 테스트
│   │   │   ├── api.ts                        # MOD: 계약·양식·고객·당사자·일정 API 함수 추가
│   │   │   └── nav.ts                        # MOD: 계약 관리·고객 관리·일정 그룹 추가
│   │   ├── components/
│   │   │   ├── properties/
│   │   │   │   ├── party-section.tsx         # NEW: 매물 폼 내 당사자 관리
│   │   │   │   └── contract-progress.tsx     # NEW: 진행률 바(재사용)
│   │   │   ├── calendar/
│   │   │   │   ├── calendar-grid.tsx         # NEW: 월 그리드
│   │   │   │   ├── calendar-sidebar.tsx      # NEW: 필터+선택일 패널
│   │   │   │   ├── event-dialog.tsx          # NEW: 추가/수정 다이얼로그
│   │   │   │   ├── event-detail-dialog.tsx   # NEW: 상세 다이얼로그
│   │   │   │   └── day-panel.tsx             # NEW: 선택일 패널
│   │   │   └── ui/                           # 기존 프리미티브 재사용
│   │   ├── routes/
│   │   │   ├── properties/
│   │   │   │   ├── contract-page.tsx         # NEW: /properties/:id/contract
│   │   │   │   ├── print-page.tsx            # NEW: /properties/:id/contract/print/:formId
│   │   │   │   └── property-form.tsx         # MOD: PartySection 포함
│   │   │   ├── customers/
│   │   │   │   ├── list-page.tsx             # NEW: /customers
│   │   │   │   ├── form-page.tsx             # NEW: /customers/new, /customers/:id
│   │   │   │   └── customer-list.tsx         # NEW: 표/카드, 검색·필터
│   │   │   └── calendar/
│   │   │       └── calendar-page.tsx         # NEW: /calendar
│   │   ├── router.tsx                        # MOD: 새 라우트 등록
│   │   └── e2e/
│   │       └── stage2.spec.ts                # NEW: 계약·고객·일정 E2E
├── app/                                      # 변경 없음(같은 SPA 번들 사용)
└── docs/
    └── PROJECT_GUIDE.md                      # MOD: 현재 참고 상태 갱신
```

---

### Task 0: 공통 라이브러리 이식

**Files:**
- Copy from 원본 `c:\Wspace\real-es/src/lib/` → `web/src/lib/`:
  - `properties/contract-checklist.ts` (+ `contract-checklist.test.ts`)
  - `properties/contract-forms.ts` (+ `contract-forms.test.ts`)
  - `properties/party-role.ts`
  - `properties/format.ts` (+ `format.test.ts` — `formatYmd` 테스트 추가)
  - `calendar/month-grid.ts` (+ `month-grid.test.ts`)
  - `calendar/categories.ts`
  - `calendar/property-events.ts` (+ `property-events.test.ts`)
  - `customers/types.ts` (+ `types.test.ts`)
- Modify: `web/src/lib/nav.ts` (그룹 추가), `web/src/lib/api.ts` (새 API 함수 자리 확보)

**Interfaces:**
- Produces: `web/src/lib/` 아래 위 파일들. `pnpm --dir web test` 로 단위 테스트 전부 통과.

- [ ] **Step 1:** 원본 파일들을 `web/src/lib/` 대응 경로에 복사. Next 전용 import(`next/navigation` 등) 없음 확인.
- [ ] **Step 2:** `format.ts` 에 `formatYmd(value: string): string` 추가 — `"YYYYMMDD"` → `"M월 D일"`. 테스트 케이스 추가.
- [ ] **Step 3:** `nav.ts` 에 spec §7.7 그룹 3개(계약 관리, 고객 관리, 일정) 추가. `allNavItems` 자동 반영 확인.
- [ ] **Step 4:** `api.ts` 에 아래 함수 시그니처만 먼저 선언(구현은 각 Task에서):
  - `getContract(propertyId)`, `toggleChecklist(propertyId, itemId, checked)`, `checklistAll(propertyId, checked)`, `startContract(propertyId)`, `completeContract(propertyId)`
  - `getForms(propertyId)`, `getFormHtml(propertyId, formId)`
  - `listCustomers(q?, type?)`, `getCustomer(id)`, `createCustomer(input)`, `updateCustomer(id, input)`, `deleteCustomer(id)`
  - `listParties(propertyId)`, `addParty(propertyId, role, name, phone)`, `removeParty(propertyId, partyId)`
  - `loadCalendar(year, month)`, `createEvent(input)`, `updateEvent(id, input)`, `deleteEvent(id)`
- [ ] **Step 5:** `pnpm --dir web test` → 전체 통과. `tsc --noEmit` 통과.
- [ ] **Step 6:** Commit — `feat(web): 2단계 공통 라이브러리 이식`.

---

### Task 1: 서버 — 계약 API (`/api/v1/contract`)

**Files:**
- Create: `server/src/contract/{mod,model,repo,handlers,validate}.rs`, `server/tests/contract.rs`
- Modify: `server/src/lib.rs` (라우트 등록)

**Interfaces:**
- Consumes: `CurrentUser`, `ApiError`, `notify_superadmins`
- Produces:
  - `GET /api/v1/contract/{propertyId}` → `ContractData`:
    ```rust
    struct ContractData {
        property: PropertyDto,
        checklist: Vec<ChecklistItem>,
        checked: HashMap<String, String>,  // itemId -> ISO timestamp
        required_fields: Vec<RequiredField>, // {key, label, filled: bool}
        progress: Progress,                // {done, total, complete}
        status: String,
    }
    ```
  - `PATCH /api/v1/contract/{propertyId}/checklist/{itemId}` 본문 `{checked: bool}` → 204. 유효하지 않은 itemId 400.
  - `POST /api/v1/contract/{propertyId}/checklist/all` 본문 `{checked: bool}` → 204.
  - `POST /api/v1/contract/{propertyId}/start` → 200 `ContractData`. `status: "진행" → "계약진행"`, 알림 발송.
  - `POST /api/v1/contract/{propertyId}/complete` → 200 `ContractData`. `progress.complete` 검증, 미충족 400. 알림 발송.

- [ ] **Step 1: 실패하는 테스트** `server/tests/contract.rs` (`common::signup`, `common::create_property` 헬퍼 사용):
  - `get_contract_returns_checklist_and_progress` — 매물 생성 → `GET /contract/{id}` → `checklist` 길이 > 0, `progress.total > 0`, `status == "진행"`.
  - `toggle_checklist_item_updates_checked_map` — `PATCH /checklist/{itemId} {checked:true}` → `checked` 에 ISO timestamp 추가. `checked:false` → 키 삭제.
  - `toggle_invalid_item_id_is_400` — 존재하지 않는 itemId → 400.
  - `checklist_all_true_sets_all_items` — `POST /checklist/all {checked:true}` → 해당 유형 모든 항목 `checked` 에 들어감.
  - `checklist_all_false_clears_all` — `checked:false` → `checked` 빈 객체.
  - `start_contract_changes_status_and_notifies` — `POST /start` → `status == "계약진행"`, `Notification` 행 증가(`type="contract"`).
  - `start_contract_non_progress_is_400` — 이미 `계약진행`/`계약완료`면 400.
  - `complete_contract_requires_full_progress` — 필수 체크·필드 미충족 → 400 `"필수 항목·데이터가 모두 충족되지 않았습니다"`.
  - `complete_contract_changes_status_and_notifies` — 모두 충족 시 `status == "계약완료"`, 알림 발송.
  - `complete_from_progress_not_contract_progress_is_400` — `status != "계약진행"` 이면 400.
  - `other_users_contract_is_404` — 사용자 B가 A의 매물 계약 API 호출 → 404.
- [ ] **Step 2:** `cargo test -p resm-server --test contract` → FAIL 확인.
- [ ] **Step 3:** `validate.rs` — `resolveChecklist`, `requiredFieldKeys`, `contractProgress` 를 **원본 TS 로직과 동일하게** Rust로 포팅. 단위 테스트로 TS 버전과 같은 입력→같은 출력 검증.
- [ ] **Step 4:** `model.rs`(DTO, 검증), `repo.rs`(SQL), `handlers.rs` 구현. `contractChecklist` JSONB는 `serde_json::Value` 로 읽고 `HashMap<String, String>` 로 파싱/직렬화.
- [ ] **Step 5:** `cargo test -p resm-server` → PASS. `cargo clippy -p resm-server -- -D warnings` 통과.
- [ ] **Step 6:** Commit — `feat(server): 계약 API(체크리스트·상태 전환·알림)`.

---

### Task 2: 서버 — 양식 인쇄 API (`/api/v1/forms`)

**Files:**
- Create: `server/src/forms/{mod,templates,engine,handlers}.rs`, `server/tests/forms.rs`
- Modify: `server/src/lib.rs` (템플릿 엔진 state 추가, 라우트 등록)
- Template files: `templates/forms/{sale_contract,lease_contract,confirm_residential,confirm_nonresidential,receipt,power_of_attorney}.html` + 파셜(`property_table.html`, `amount_rows.html`, `articles.html`, `special_terms.html`, `signatures.html`, `date_line.html`, `base.html`)

**Interfaces:**
- Consumes: `CurrentUser`, `ApiError`, `formsFor` 로직(Rust 포팅)
- Produces:
  - `GET /api/v1/forms/{propertyId}` → `[{id, label}]` (적용 가능한 양식 목록)
  - `GET /api/v1/forms/{propertyId}/{formId}` → `Content-Type: text/html; charset=utf-8`, 본문 = A4 인쇄용 HTML. 슬롯 값은 `Property` 필드(`sourceKey`)에서 채움. `sourceKey` 없는 슬롯은 `<span class="fill">`(점선 밑줄).

**템플릿 엔진:** `tera` 권장(직관적, Jinja2 호환). `AppState` 에 `tera::Tera` 포함.

- [ ] **Step 1:** `tera` 크레이트 추가. `engine.rs` 에서 `Tera::new("templates/**/*")` 로 컴파일 시점 로드. 템플릿 파일들을 `server/src/forms/templates/` 에 배치.
- [ ] **Step 2: 실패하는 테스트** `server/tests/forms.rs`:
  - `forms_list_returns_applicable_forms` — 매물 `realEstateType=A01, tradeType=A1` → `sale_contract`, `confirm_residential`, `receipt`, `power_of_attorney` 4개 반환.
  - `form_html_contains_property_data` — `GET /forms/{id}/sale_contract` → 응답 HTML에 매물 주소·단지명·매매대금(`price`)가 기입돼 있음.
  - `form_html_has_empty_fills_for_missing_slots` — `sourceKey` 없는 슬롯(계약금, 잔금 등)은 빈 기입란(`<span class="fill">`)으로 렌더.
  - `form_html_has_a4_page_style` — 응답에 `@page { size: 210mm 297mm; margin: 18mm 14mm; }` 포함.
  - `form_html_uses_serif_font` — `font-family: 'Noto Serif KR', serif` 또는 시스템 serif 스택 포함.
  - `unknown_form_id_is_404` — 존재하지 않는 formId → 404.
  - `other_users_property_form_is_404` — 타인 매물 → 404.
- [ ] **Step 3:** `cargo test -p resm-server --test forms` → FAIL.
- [ ] **Step 4:** 템플릿 6종 + 파셜 작성. 원본 `contract-document.tsx` 의 `ContractDocument`, `SaleContract`, `LeaseContract`, `ConfirmStatement`, `Receipt`, `PowerOfAttorney`, `PropertyTable`, `AmountRow`, `Articles`, `SpecialTerms`, `Signatures`, `DateLine`, `Section`, `Fill` 컴포넌트를 tera 템플릿으로 1:1 변환. CSS는 `<style>` 블록에 인라인(Tailwind 클래스 대신 순수 CSS, `mm` 단위, `border-collapse: collapse`).
- [ ] **Step 5:** `handlers.rs` 구현. `formsFor` 로직 Rust 포팅(`contract-forms.ts` 와 동일). `Property` 조회 → 템플릿 렌더 → `Html(String)` 반환.
- [ ] **Step 6:** `cargo test -p resm-server` → PASS.
- [ ] **Step 7:** Commit — `feat(server): 양식 인쇄 API(HTML 템플릿)`.

---

### Task 3: 서버 — 고객 API (`/api/v1/customers`)

**Files:**
- Create: `server/src/customers/{mod,model,repo,handlers}.rs`, `server/tests/customers.rs`
- Modify: `server/src/lib.rs`

**Interfaces:**
- `GET /api/v1/customers?q=&type=` → `CustomerRow[]`
- `GET /api/v1/customers/{id}` → `CustomerRow` (parties 포함)
- `POST /api/v1/customers` 본문 `CustomerInput` → 201 `{id}`. `types` 정규화, `propertyId` 연결 시 당사자 upsert.
- `PATCH /api/v1/customers/{id}` 본문 `CustomerInput` (선택적) → 204.
- `DELETE /api/v1/customers/{id}` → 204.

**`CustomerInput`**: `name`, `zipcode?`, `phone?`, `address?`, `email?`, `gender?`, `memo?`, `types?: string[]`, `propertyId?`

**`CustomerRow`**: `id`, `types:string[]`, `name`, `zipcode|null`, `phone|null`, `address|null`, `email|null`, `gender|null`, `memo|null`, `propertyId|null`, `propertyName|null`, `parties: {propertyId, propertyLabel, role}[]`, `updatedAt: string(YYYY-MM-DD)`

- [ ] **Step 1: 실패하는 테스트** `server/tests/customers.rs`:
  - `list_customers_search_and_type_filter` — 이름·전화 부분일치(`q`), 유형 필터(`type=매도인` 등) 동작.
  - `create_customer_normalizes_types` — `types: ["매도인", "단순방문"]` → 정규화 후 `["매도인"]` 저장. 빈 배열 → `["단순방문"]`.
  - `create_customer_with_property_id_links_party` — `propertyId` 전달 → `PropertyParty` upsert(역할=`ownerRoleFromTrade(tradeType)`).
  - `update_customer_changes_property_relink` — `propertyId` 변경 시 기존 당사자 해제·새 연결.
  - `delete_customer_removes_owned_only` — 타인 고객 삭제 시도 → 404(행 변화 없음).
  - `gender_validation` — `gender: "기타"` → 400. 유효값만 허용.
  - `name_required` — 이름 없음 → 400.
- [ ] **Step 2:** FAIL 확인 → 구현 → PASS.
- [ ] **Step 3:** `normalizeCustomerTypes`, `ownerRoleFromTrade` Rust 포팅(`model.rs` 또는 `validate.rs` 공용).
- [ ] **Step 4:** Commit — `feat(server): 고객 API`.

---

### Task 4: 서버 — 당사자 API (`/api/v1/properties/{propertyId}/parties`)

**Files:**
- Create: `server/src/parties/{mod,model,repo,handlers}.rs`, `server/tests/parties.rs`
- Modify: `server/src/lib.rs`

**Interfaces:**
- `GET /api/v1/properties/{propertyId}/parties` → `PartyRow[]` (`id`, `customerId`, `name`, `phone`, `role`)
- `POST /api/v1/properties/{propertyId}/parties` 본문 `{role, name, phone}` → 201 `{partyId, customerId}`. **이름+전화(숫자만)로 기존 고객 검색** → 재사용, 없으면 생성(`types=[role]`). `PropertyParty` upsert.
- `DELETE /api/v1/properties/{propertyId}/parties/{partyId}` → 204.

**매칭 키**: `custKey = trim(name) | digits_only(phone)` — 원본 `party-actions.ts` 와 동일.

- [ ] **Step 1: 실패하는 테스트** `server/tests/parties.rs`:
  - `add_party_reuses_existing_customer` — 같은 이름+전화 고객이 있으면 재사용(`customerId` 동일), `types` 에 역할 추가(중복 없음).
  - `add_party_creates_new_customer` — 매칭 고객 없으면 신규 생성(`types=[role]`).
  - `add_party_upserts_property_party` — 같은 `propertyId, customerId, role` 로 두 번 호출 → 1건만 생성.
  - `add_party_invalid_role_400` — `role: "기타"` → 400.
  - `add_party_name_required` — 이름 없음 → 400.
  - `list_parties_returns_role_order` — 응답 순서: 매도인, 매수인, 임대인, 임차인, 중개.
  - `remove_party_deletes_link_only` — `PropertyParty` 삭제, `Customer` 는 유지.
  - `other_users_property_parties_404` — 타인 매물 → 404.
- [ ] **Step 2:** FAIL → 구현 → PASS.
- [ ] **Step 3:** Commit — `feat(server): 당사자 API(자동 매칭·업서트)`.

---

### Task 5: 서버 — 일정 API (`/api/v1/events`)

**Files:**
- Create: `server/src/events/{mod,model,repo,handlers}.rs`, `server/tests/events.rs`
- Modify: `server/src/lib.rs`

**Interfaces:**
- `GET /api/v1/events?year=2026&month=11` → `{ events, propertyDates, properties, customers }`
- `POST /api/v1/events` 본문 `EventInput` → 201 `{id}`
- `PATCH /api/v1/events/{id}` 본문 `EventInput` (선택적) → 204
- `DELETE /api/v1/events/{id}` → 204

**`EventInput`**: `title`, `date(YYYYMMDD)`, `startTime?`, `category`, `memo?`, `propertyId?`, `customerId?`

**검증**: 제목·날짜 필수. `category ∈ ["미팅","임장","계약","기타"]`. `propertyId`, `customerId` 본인 것 확인.

**파생 일정**: `Property` 의 7개 날짜 필드에서 `propertyCalendarEvents` 로 생성(원본 TS 포팅). 월 접두사(`YYYYMM`)로 필터.

- [ ] **Step 1: 실패하는 테스트** `server/tests/events.rs`:
  - `load_calendar_returns_events_and_property_dates` — 수기 일정 + 매물 파생 일정 모두 반환. `propertyDates` 에 `label`, `propertyId`, `field` 포함.
  - `load_calendar_filters_by_month` — `year=2026,month=11` → 202611 접두사만.
  - `create_event_validates_category` — `category: "기타"` 외 값 → 400.
  - `create_event_links_property_customer` — 본인 `propertyId`/`customerId` 연결 저장.
  - `update_event_changes_fields` — 부분 수정 동작.
  - `delete_event_removes_owned_only` — 타인 일정 삭제 시도 → 404.
  - `property_dates_derived_from_property_fields` — 매물에 `contractDate=20261115` 설정 → `propertyDates` 에 `label="매물명 계약일"` 포함.
- [ ] **Step 2:** `propertyCalendarEvents`, `monthPrefix`, `buildMonthGrid` 관련 로직 Rust 포팅(`validate.rs` 공용).
- [ ] **Step 3:** FAIL → 구현 → PASS.
- [ ] **Step 4:** Commit — `feat(server): 일정 API(수기+파생)`.

---

### Task 6: 화면 — 계약 페이지 (`/properties/:id/contract`)

**Files:**
- Create: `web/src/routes/properties/contract-page.tsx`, `web/src/components/properties/contract-progress.tsx`
- Copy from 원본: `src/app/(dashboard)/properties/[id]/contract/contract-client.tsx` → 참고해 이식
- Modify: `web/src/routes/properties/property-form.tsx` (PartySection import), `web/src/lib/api.ts` (계약 함수 구현)

**Interfaces:**
- Consumes: `getContract`, `toggleChecklist`, `checklistAll`, `startContract`, `completeContract`, `updateProperty`(핵심 필드 저장), `ContractData`, `ChecklistItem`, `Progress`

- [ ] **Step 1:** 원본 `ContractClient` 를 SPA용으로 변환:
  - `useRouter`/`useSearchParams` → `react-router` `useNavigate`/`useSearchParams`
  - 서버 액션 호출 → `api.ts` 함수
  - `toast` → 기존 `sonner` 사용
  - 낙관적 업데이트: `useMutation` `onMutate`/`onError` 로 `checked`, `requiredFields.filled`, `progress` 즉시 반영, 실패 시 롤백
- [ ] **Step 2:** `contract-progress.tsx` — 진행률 바(원본 스타일: `h-2 w-full rounded-full bg-primary`).
- [ ] **Step 3:** `contract-page.tsx` — 레이아웃: 경고 배너 → 매물 헤더 → 진행률 → 핵심 데이터(그리드) → 체크리스트(그룹별) → 양식 버튼들 → 하단 액션(계약진행/계약완료 버튼).
- [ ] **Step 4:** `api.ts` 계약 함수들 구현. 쿼리 키 `["contract", propertyId]`. 변경 시 `["contract", propertyId]`, `["property", propertyId]`, `["properties"]` 무효화.
- [ ] **Step 5:** 라우터에 `/properties/:id/contract` 등록. `RequireAuth` 보호.
- [ ] **Step 6:** 브라우저 수동 확인 — 체크 토글, 진행률 갱신, 계약진행 전환, 필수 미충족 시 완료 버튼 비활성, 완료 처리 후 목록으로 이동. 원본 웹과 비교.
- [ ] **Step 7:** `tsc --noEmit`, `pnpm --dir web test`, `pnpm --dir web build` 통과.
- [ ] **Step 8:** Commit — `feat(web): 계약 페이지(체크리스트·진행률·상태 전환)`.

---

### Task 7: 화면 — 양식 인쇄 페이지 (`/properties/:id/contract/print/:formId`)

**Files:**
- Create: `web/src/routes/properties/print-page.tsx`
- Modify: `web/src/lib/api.ts` (`getFormHtml` 구현), `web/src/routes/properties/contract-page.tsx` (버튼 `target="_blank"`)

**Interfaces:**
- `getFormHtml(propertyId, formId)` → `Promise<string>` (HTML 텍스트)
- 페이지 마운트 시 `dangerouslySetInnerHTML` 로 렌더 + `window.print()` 호출.

- [ ] **Step 1:** `print-page.tsx` — `useEffect` 에서 `getFormHtml` 호출, 상태에 HTML 저장, 렌더 후 `window.print()`.
- [ ] **Step 2:** `api.ts` `getFormHtml` — `fetch` 로 HTML 텍스트 받기(`Accept: text/html`).
- [ ] **Step 3:** 계약 페이지 양식 버튼에 `target="_blank" rel="noopener"` 추가.
- [ ] **Step 4:** 브라우저 수동 확인 — 6종 양식 버튼 클릭 → 새 탭 A4 렌더 → 인쇄 대화상자 → 데이터 자동 기입(주소, 단지명, 금액, 당사자명 등) 확인. 원본 웹 인쇄 미리보기와 시각 비교.
- [ ] **Step 5:** Android 웹뷰에서 `window.print()` 동작 확인(안 되면 "PDF 저장" 버튼 대체 — 나중 Task).
- [ ] **Step 6:** Commit — `feat(web): 양식 인쇄 페이지`.

---

### Task 8: 화면 — 고객 페이지 (`/customers`, `/customers/new`, `/customers/:id`)

**Files:**
- Create: `web/src/routes/customers/{list-page,form-page}.tsx`, `web/src/routes/customers/customer-list.tsx`
- Copy from 원본: `src/app/(dashboard)/customers/{customer-list,customer-form}.tsx` → 참고해 이식
- Modify: `web/src/lib/api.ts` (고객 함수들 구현), `web/src/router.tsx` (라우트 등록)

**Interfaces:**
- `listCustomers(q?, type?)`, `getCustomer(id)`, `createCustomer(input)`, `updateCustomer(id, input)`, `deleteCustomer(id)`
- 쿼리 키 `["customers", q, type]`, `["customer", id]`. 변경 시 `["customers"]` 무효화.

- [ ] **Step 1:** `customer-list.tsx` 이식 — 검색 input, 유형 Select, 표/카드 반응형, 빈 상태, 삭제 확인 다이얼로그.
- [ ] **Step 2:** `form-page.tsx` 이식 — 고객유형 칩 토글(단순방문 단독), 이름/전화/이메일/성별/우편번호/주소/메모. `PostcodeSearch` 재사용. 매물에서 열린 경우(`propertyId`, `propertyLabel` 쿼리) 상단 표시.
- [ ] **Step 3:** `list-page.tsx` — `<CustomerList>` 래핑. `form-page.tsx` — `customerId` 유무로 신규/수정 분기.
- [ ] **Step 4:** `api.ts` 고객 함수들 구현.
- [ ] **Step 5:** 라우터 등록: `/customers`, `/customers/new`, `/customers/:id`. `RequireAuth`.
- [ ] **Step 6:** 브라우저 수동 확인 — 검색·필터, 등록·수정·삭제, 우편번호 검색, 매물에서 고객 등록 시 당사자 자동 연결. 원본 웹과 비교.
- [ ] **Step 7:** Commit — `feat(web): 고객 목록·등록·수정·삭제`.

---

### Task 9: 화면 — 당사자 섹션 (매물 수정 폼 내)

**Files:**
- Create: `web/src/components/properties/party-section.tsx`
- Modify: `web/src/routes/properties/property-form.tsx` (PartySection 포함), `web/src/lib/api.ts` (당사자 함수들)

**Interfaces:**
- `listParties(propertyId)`, `addParty(propertyId, role, name, phone)`, `removeParty(propertyId, partyId)`
- 다이얼로그: 역할 Select, 이름(필수), 전화. 저장 시 `addParty` → 성공하면 `["parties", propertyId]`, `["customers"]` 무효화.

- [ ] **Step 1:** `party-section.tsx` — 현재 당사자 목록(역할·이름·전화·삭제), "당사자 추가" 버튼 → 다이얼로그.
- [ ] **Step 2:** `property-form.tsx` 에 `<PartySection propertyId={id} />` 추가(원본 위치: 핵심 데이터 아래, 메모 위).
- [ ] **Step 3:** `api.ts` 당사자 함수들 구현.
- [ ] **Step 4:** 브라우저 수동 확인 — 당사자 추가 시 기존 고객 매칭(이름+전화), 신규 생성, 삭제, 고객 목록에 자동 반영.
- [ ] **Step 5:** Commit — `feat(web): 매물 당사자 섹션`.

---

### Task 10: 화면 — 일정 페이지 (`/calendar`)

**Files:**
- Create: `web/src/routes/calendar/calendar-page.tsx`, `web/src/components/calendar/{calendar-grid,calendar-sidebar,event-dialog,event-detail-dialog,day-panel}.tsx`
- Copy from 원본: `src/app/(dashboard)/calendar/{calendar-view,event-dialog,event-detail-dialog}.tsx` → 참고해 이식
- Modify: `web/src/lib/api.ts` (일정 함수들), `web/src/router.tsx`

**Interfaces:**
- `loadCalendar(year, month)`, `createEvent(input)`, `updateEvent(id, input)`, `deleteEvent(id)`
- 쿼리 키 `["events", year, month]`. 변경 시 무효화.
- URL 동기화: `useSearchParams`로 `y`, `m`, `d` 읽기/쓰기(`history.replaceState`).

- [ ] **Step 1:** `calendar-grid.tsx` — `buildMonthGrid` 로 6주×7일 렌더. 당월/전월/다음달 클래스 구분. 셀 클릭 → 선택일 변경. 수기 일정(색상 점) + 매물 파생 일정(빨간 점) 최대 3개 표시, 초과 `+N`.
- [ ] **Step 2:** `calendar-sidebar.tsx` — 유형 체크박스(미팅/임장/계약/기타 + 매물 일정). 선택일 패널(`day-panel.tsx`): 수기 일정 클릭 → 상세 다이얼로그, 매물 파생 클릭 → 매물 수정 폼 이동.
- [ ] **Step 3:** `event-dialog.tsx` — 추가/수정 폼(제목 필수, 날짜, 시작시간, 카테고리 라디오, 메모, 매물/고객 선택). `EventDetailDialog` 에서 "수정" 버튼으로 전환.
- [ ] **Step 4:** `calendar-page.tsx` — 전체 조립. 월 이동 버튼, 오늘 버튼, 일정 추가 버튼. 로딩 상태 표시. URL 동기화(`y`, `m`, `d`).
- [ ] **Step 5:** `api.ts` 일정 함수들 구현.
- [ ] **Step 6:** 라우터 등록 `/calendar`. `RequireAuth`.
- [ ] **Step 7:** 브라우저 수동 확인 — 월 이동, 수기 일정 CRUD, 카테고리 필터, 매물 파생 일정 표시/클릭 이동, URL 복원(뒤로가기). 원본 웹과 비교.
- [ ] **Step 8:** Commit — `feat(web): 일정 캘린더(월 그리드·필터·다이얼로그)`.

---

### Task 11: E2E 스모크 확장

**Files:**
- Create: `web/e2e/stage2.spec.ts`
- Modify: `web/playwright.config.ts` (필요시)

**시나리오:**
1. 로그인 → 매물 등록 → 계약 페이지 진입 → 체크리스트 토글 → 진행률 확인 → 계약진행 전환 → 필수 필드 채우기 → 계약완료 처리 → 목록에서 `계약완료` 뷰 확인.
2. 양식 인쇄: 계약 페이지에서 `매매계약서` 버튼 클릭 → 새 탭 HTML 렌더 → 데이터 기입 확인.
3. 고객: 목록 → 검색·필터 → 새 고객 등록(우편번호 검색 포함) → 수정 → 삭제.
4. 당사자: 매물 수정 폼에서 당사자 추가(기존 고객 매칭/신규 생성) → 삭제 → 고객 목록 반영 확인.
5. 일정: 캘린더 월 이동 → 수기 일정 추가/수정/삭제 → 카테고리 필터 토글 → 매물 파생 일정 클릭 → 매물 수정 폼 이동 → URL `y/m/d` 복원 확인(뒤로가기).

- [ ] **Step 1:** `stage2.spec.ts` 작성. 각 시나리오 독립적(`test.describe.serial` 아님).
- [ ] **Step 2:** `pnpm build:web && pnpm test:e2e` → PASS.
- [ ] **Step 3:** Commit — `feat(e2e): 2단계 스모크(계약·고객·일정)`.

---

### Task 12: 앱·배포 재확인

**Files:**
- Modify: `README.md`, `docs/PROJECT_GUIDE.md`, `infra/` (필요시)

- [ ] **Step 1:** `pnpm app:dev` — Windows 앱에서 계약·고객·일정 메뉴 진입, 전체 플로우 확인.
- [ ] **Step 2:** `pnpm app:android:dev` — 폰/태블릿 에뮬레이터에서 계약 페이지(카드/표), 고객(카드), 일정(사이드바 아래) 반응형 확인. 우편번호 검색, 인쇄(`window.print()`) 동작 확인.
- [ ] **Step 3:** `pnpm app:build:win`, `pnpm app:build:android` — 빌드 성공 확인.
- [ ] **Step 4:** Zorin 재배포 절차 사용자 확인 → 배포 → `https://resm2.approid.team` 에서 전 플랫폼 검증(1단계 Task 12 §5.1 기준 + 2단계 새 화면).
- [ ] **Step 5:** `README.md` "현재 반영 상태", `docs/PROJECT_GUIDE.md` "현재 참고 상태"에 2단계 결과 기록.
- [ ] **Step 6:** Commit — `feat(deploy): 2단계 배포 및 완료 기록`.

---

## 승인 대기

이 계획 검토 후 **[승인/수정 요청]** 을 주면 Task 0부터 순차 구현에 들어간다.