# RESM 2단계 설계 — 계약 워크플로·양식 인쇄, 고객·당사자, 일정

작성일: 2026-11-15
대상 저장소: `c:\Wspace\resm`
원본 저장소: `c:\Wspace\real-es` (읽기 전용 참고. 이 작업에서 수정하지 않는다)

표기: **[결정]** 은 사용자가 정한 것, **[판단]** 은 설계자가 고른 기본값이다. [판단]은 사용자가 뒤집을 수 있다.

---

## 1. 의도

1단계(뼈대: 인증 + 매물 CRUD)가 세 플랫폼(브라우저·Windows·Android)에서 동작하고 `https://resm2.approid.team` 로 배포 완료됐다.

2단계는 **실무 핵심 도구 3종**을 추가한다:
1. **계약 워크플로 + 양식 인쇄** — 매물을 `진행 → 계약진행 → 계약완료` 로 단계 전환, 체크리스트·필수 필드 검증, 6종 계약 양식 A4 인쇄
2. **고객·당사자 관리** — 고객 목록/등록/수정/삭제, 매물 ↔ 고객 다대다(역할 포함), 유형별 필터
3. **일정 관리** — 월간 캘린더, 수기 일정 CRUD(미팅·임장·계약·기타), 매물 날짜 필드 파생 일정, 카테고리 필터

모두 기존 DB 스키마(`Property.contractChecklist`, `Customer`, `PropertyParty`, `Event`)와 **100% 호환**되게 API·화면을 구현한다. 기존 웹(`resm.approid.team`)과 새 앱이 같은 데이터를 보고 쓴다.

---

## 2. 목표 / 비목표 / 반목표

### 목표
- [결정] 기존 `real-es` 는 그대로 두고 새 저장소에서 작업한다.
- [결정] 세 플랫폼(브라우저·Windows·Android) 모두에서 계약·고객·일정 화면이 동작한다.
- [결정] 화면 디자인은 현재 것을 유지(SDTPL_ADM 템플릿 기반 shadcn UI).
- [결정] DB 스키마는 **원본 Prisma가 주인**. 새 서버는 마이그레이션을 실행하지 않는다.
- [판단] 계약 체크리스트·양식 정의·고객 유형·일정 카테고리는 **원본 단일 소스 파일**을 `web/src/lib/` 로 복사해 쓰고, 서버는 검증 로직만 재구현한다(공통 로직 중복 금지).

### 비목표 (2단계)
- 엑셀 입출력, 실거래가(통계·지도) — 3단계
- 네이버 매물 수집 — 4단계
- 설정·회원·알림 화면·홈 대시보드 — 5단계
- DB 스키마 변경
- iOS·macOS·Linux 빌드

### 반목표 (동작해도 실패로 본다)
- 기존 웹(`resm.approid.team`)이나 운영 DB의 데이터가 깨지는 것.
- 새 서버가 DB 스키마를 바꿔 Prisma 마이그레이션 이력과 어긋나는 것.
- 계약 상태·체크리스트·당사자·일정 데이터가 기존 웹과 새 앱에서 다르게 보이거나 저장되는 것.
- 인쇄용 A4 양식이 기존 웹과 레이아웃·데이터 매핑이 다른 것.

---

## 3. 전체 구조 (1단계에서 확장)

```
┌─────────────── 화면 (React SPA, 한 벌) ───────────────┐
│  Windows 앱(Tauri)   Android 앱(Tauri)   브라우저      │
└───────────────┬───────────────┬───────────────┬───────┘
                │  HTTPS JSON (/api/v1)          │
                ▼                                ▼
         ┌────────────── Rust API 서버 (axum) ─────────────┐
         │  인증 · 매물 · 계약·체크리스트 · 양식 · 고객 · 당사자 · 일정 · SPA 정적 제공 │
         └───────────────────────┬─────────────────────────┘
                                 │ sqlx
                                 ▼
                      기존 PostgreSQL `real_es` (스키마 그대로)
                                 ▲
                                 │ Prisma (전환 기간에만 병행)
                      기존 Next.js 웹 `resm.approid.team`
```

---

## 4. 2단계 로드맵 (세부 Task)

| Task | 내용 | 산출물 |
|---|---|---|
| 0 | 공통 라이브러리 이식: `contract-checklist`, `contract-forms`, `party-role`, `calendar/*`, `customers/types`, `format`(날짜·전화 포맷 추가) | `web/src/lib/properties/`, `web/src/lib/calendar/`, `web/src/lib/customers/` |
| 1 | 서버: 계약 API — 상태 전환(`진행→계약진행`, `계약완료`), 체크리스트 CRUD, 필수 필드 검증 | `server/src/contract/` |
| 2 | 서버: 양식 인쇄 API — HTML 반환(프린트용), 6종 양식·슬롯 매핑 | `server/src/forms/` |
| 3 | 서버: 고객 API — 목록·단건·생성·수정·삭제, 유형 필터 | `server/src/customers/` |
| 4 | 서버: 당사자 API — 매물 당사자 목록·추가·삭제(이름+전화로 고객 자동 매칭/생성) | `server/src/parties/` |
| 5 | 서버: 일정 API — 월간 조회(수기+파생), 생성·수정·삭제, 카테고리 검증 | `server/src/events/` |
| 6 | 화면: 계약 페이지 — 진행률·핵심데이터·체크리스트·양식 버튼·완료 처리 | `web/src/routes/properties/[id]/contract/` |
| 7 | 화면: 양식 인쇄 페이지 — 새 탭 A4 렌더, `window.print()` | `web/src/routes/properties/[id]/contract/print/[formId]/` |
| 8 | 화면: 고객 페이지 — 목록(표/카드), 검색·유형 필터, 등록·수정 폼(우편번호 포함) | `web/src/routes/customers/` |
| 9 | 화면: 당사자 섹션 — 매물 수정 폼 내 당사자 추가/삭제, 고객 자동 연결 | `web/src/components/properties/party-section.tsx` |
| 10 | 화면: 일정 페이지 — 월 그리드, 사이드바 필터, 선택일 패널, 추가/수정/상세 다이얼로그 | `web/src/routes/calendar/` |
| 11 | 네비게이션·사이드바 확장 — "계약 관리"(계약 진행/완료 뷰는 기존), "고객 관리", "일정" 그룹 추가 | `web/src/lib/nav.ts` |
| 12 | E2E 스모크 확장 — 계약 워크플로, 고객 CRUD, 일정 CRUD | `web/e2e/stage2.spec.ts` |
| 13 | 앱·배포 재확인 — 세 플랫폼에서 새 화면 동작, Zorin 재배포 | - |

각 Task는 1단계 plan과 같은 체크리스트 형식으로 상세화한다.

---

## 5. 공통 라이브러리 이식 (Task 0)

원본 `c:\Wspace\real-es/src/lib/` 에서 아래 파일들을 **그대로 복사**해 `web/src/lib/` 에 둔다. Next 전용 import만 제거(`next/navigation` 등 없음).

### 5.1 `web/src/lib/properties/contract-checklist.ts`
- `ChecklistItem`, `PARTIES`, `PARTY_LABEL`, `GROUPS`, `resolveChecklist(realEstateType, tradeType)`, `requiredFieldKeys(tradeType)`, `contractProgress(...)`
- **서버도 이 로직을 재사용**해야 하므로 순수 TS(런타임 의존 없음)로 둔다. 서버 Rust 측은 동일 로직을 포팅해 검증에 쓴다.

### 5.2 `web/src/lib/properties/contract-forms.ts`
- `ContractForm`, `FormFieldSlot`, `CONTRACT_FORMS`, `FORM_BY_ID`, `formsFor(realEstateType, tradeType)`
- 슬롯 ↔ Property 키 매핑(`sourceKey`) 포함. 인쇄 API가 이 매핑으로 자동 기입.

### 5.3 `web/src/lib/properties/party-role.ts`
- `ownerRoleFromTrade(tradeType)` — 매물 등록 시 당사자 기본 역할 결정.

### 5.4 `web/src/lib/calendar/` 전체
- `month-grid.ts`: `buildMonthGrid`, `ymd`, `monthPrefix`, `MONTH_NAMES_KO`, `WEEKDAYS_KO`
- `categories.ts`: `CALENDAR_CATEGORIES`(미팅/임장/계약/기타), `PROPERTY_EVENT_STYLE`, `categoryStyle`
- `property-events.ts`: `PROPERTY_DATE_FIELDS`, `propertyCalendarEvents`

### 5.5 `web/src/lib/customers/types.ts`
- `CUSTOMER_TYPES`, `PARTY_TYPES`, `GENDERS`, `normalizeCustomerTypes`

### 5.6 `web/src/lib/properties/format.ts` 확장
- 기존 `formatTel`, `toDateInput`, `fromDateInput`, `groupDigits`, `stripDigits` 유지
- 추가: `formatYmd(value: string): string` → `"M월 D일"` (캘린더 헤더용)

### 5.7 단위 테스트도 복사
- `contract-checklist.test.ts`, `contract-forms.test.ts`, `property-events.test.ts`, `month-grid.test.ts`, `customer-types.test.ts` → `web/src/lib/` 해당 위치에 두고 `pnpm --dir web test` 로 통과 확인.

---

## 6. 서버 API 설계

공통 규칙(1단계와 동일):
- 베이스 경로 `/api/v1`
- 모든 쿼리에 `"userId" = 현재 사용자` 조건. 남의 데이터는 404.
- 금액은 **원 단위 십진 문자열**. 날짜는 `YYYYMMDD` 문자열.
- 오류 응답: `{ "error": "한국어 메시지" }`. 400/401/403/404/500.
- `contractChecklist` 컬럼은 JSONB(`Record<string, string>` — itemId → ISO timestamp).

### 6.1 계약 API (`/api/v1/contract`)

| 메서드·경로 | 동작 |
|---|---|
| `GET /contract/{propertyId}` | 계약 데이터 조회: `property`, `checklist`(resolveChecklist 결과), `checked`(contractChecklist 파싱), `requiredFields`(필수 필드 키·라벨·채워짐), `progress`(done/total/complete), `status` |
| `PATCH /contract/{propertyId}/checklist/{itemId}` | 체크 토글: `checked: boolean` → `contractChecklist` JSONB 업데이트. 유효하지 않은 itemId면 400. |
| `POST /contract/{propertyId}/checklist/all` | 전체 체크/해제: `{ checked: boolean }` → 해당 매물 유형의 모든 항목 일괄 처리. |
| `POST /contract/{propertyId}/start` | `status: "진행" → "계약진행"` 전환. `notify_superadmins("contract", "매물 계약진행: {label}", undefined, "/properties/{id}/edit")`. 이미 `계약진행`/`계약완료`면 400. |
| `POST /contract/{propertyId}/complete` | `status: "계약진행" → "계약완료"` 전환. `contractProgress` 로 필수 체크·필드 모두 충족 확인. 미충족 시 400 `"필수 항목·데이터가 모두 충족되지 않았습니다"`. 성공 시 `notify_superadmins("contract", "매물 계약완료: {label}", ...)`. |

**응답 공통**: `{ "property": PropertyDto, "checklist": ChecklistItem[], "checked": Record<string,string>, "requiredFields": {key,label,filled}[], "progress": {done,total,complete}, "status": string }`

### 6.2 양식 인쇄 API (`/api/v1/forms`)

| 메서드·경로 | 동작 |
|---|---|
| `GET /forms/{propertyId}` | 적용 가능한 양식 목록: `formsFor(g, t)` 결과 `{id, label}[]` 반환. |
| `GET /forms/{propertyId}/{formId}` | **HTML 반환**(`Content-Type: text/html; charset=utf-8`). `ContractDocument` 와 동일 렌더링(원본 `contract-document.tsx` 로직을 Rust 템플릿으로 포팅). 슬롯 값은 `Property` 필드(`sourceKey`)에서 채움. `sourceKey` 없는 슬롯은 빈 기입란(점선). A4 `@page` 스타일 포함. 새 탭에서 열리면 브라우저가 바로 인쇄 대화상자 띄움(`window.print()`는 화면 측에서). |

**서버 사이드 렌더링 방식**: `tera`/`askama`/`handlebars` 중 하나로 HTML 템플릿 6종(`sale_contract`, `lease_contract`, `confirm_residential`, `confirm_nonresidential`, `receipt`, `power_of_attorney`) 작성. 공통 컴포넌트(부동산 표시 테이블, 금액 명세, 조문, 특약, 서명란) 파셜로 분리.

### 6.3 고객 API (`/api/v1/customers`)

| 메서드·경로 | 동작 |
|---|---|
| `GET /customers?q=&type=` | 목록: 검색 `q`(이름·전화 부분일치, 대소문자 무시), 유형 필터 `type`(ALL 또는 `CUSTOMER_TYPES` 중 하나). 정렬 `updatedAt DESC`. 응답: `CustomerRow[]` |
| `GET /customers/{id}` | 단건. `parties` 포함(매물 라벨·역할). |
| `POST /customers` | 생성. 본문 `CustomerInput`. `types` 정규화(`normalizeCustomerTypes`). `propertyId` 가 있으면 본인 매물인지 확인 후 연결. 생성 후 해당 매물의 당사자 역할(`ownerRoleFromTrade`)로 `PropertyParty` upsert. 201 `{id}`. |
| `PATCH /customers/{id}` | 수정. 본문 `CustomerInput` (필드 선택적). `types` 정규화. `propertyId` 변경 시 기존 연결 해제·새 연결. |
| `DELETE /customers/{id}` | 삭제. 본인 것만. 204. |

**`CustomerRow` 필드**: `id`, `types:string[]`, `name`, `zipcode|null`, `phone|null`, `address|null`, `email|null`, `gender|null`, `memo|null`, `propertyId|null`, `propertyName|null`, `parties: {propertyId, propertyLabel, role}[]`, `updatedAt: string(YYYY-MM-DD)`

**검증**: 이름 필수. 성별은 `GENDERS` 중 하나. 유형은 `CUSTOMER_TYPES` 중 하나. `normalizeCustomerTypes` 결과가 비면 기본 `["단순방문"]`.

### 6.4 당사자 API (`/api/v1/properties/{propertyId}/parties`)

| 메서드·경로 | 동작 |
|---|---|
| `GET /properties/{propertyId}/parties` | 당사자 목록: `{id, customerId, name, phone, role}[]`. 역할 순서: `매도인, 매수인, 임대인, 임차인, 중개` (원본 `PARTY_TYPES` + `agent`). |
| `POST /properties/{propertyId}/parties` | 당사자 추가. 본문 `{ role, name, phone }`. `role ∈ PARTY_TYPES`. 이름 필수. **이름+전화(숫자만)로 기존 고객 검색** → 있으면 재사용, 없으면 `Customer` 생성(`types=[role]`). `PropertyParty` upsert(유니크: `propertyId, customerId, role`). 201 `{partyId, customerId}`. |
| `DELETE /properties/{propertyId}/parties/{partyId}` | 당사자 삭제. `PropertyParty` 삭제. 204. |

### 6.5 일정 API (`/api/v1/events`)

| 메서드·경로 | 동작 |
|---|---|
| `GET /events?year=&month=` | 월간 조회. `year, month` 필수. 응답: `{ events: EventRow[], propertyDates: PropertyDateEvent[], properties: CalendarOption[], customers: CalendarOption[] }` |
| `POST /events` | 생성. 본문 `EventInput`. 제목·날짜 필수. `category ∈ CALENDAR_CATEGORIES` 값(미팅/임장/계약/기타). `propertyId`, `customerId` 는 본인 것 확인 후 연결. 201 `{id}`. |
| `PATCH /events/{id}` | 수정. 본문 `EventInput` (필드 선택적). 동일 검증. 204. |
| `DELETE /events/{id}` | 삭제. 204. |

**`EventRow`**: `id`, `title`, `date(YYYYMMDD)`, `startTime|null(HH:MM)`, `category`, `memo|null`, `propertyId|null`, `propertyLabel|null`, `customerId|null`, `customerLabel|null`

**`PropertyDateEvent`**: `date`, `label`, `propertyId`, `field` (원본 `propertyCalendarEvents` 결과 그대로)

**`CalendarOption`**: `{id, label}` — 일정 등록 다이얼로그용 매물·고객 선택지.

**파생 일정**: 서버가 `Property` 의 7개 날짜 필드(`contractHopeDate`…`moveInDate`)에서 `propertyCalendarEvents` 로 생성. 클라이언트도 같은 로직(`web/src/lib/calendar/property-events.ts`)을 가지지만 **권위 소스는 서버**다.

---

## 7. 화면 설계

### 7.1 라우트 추가 (기존 1단계 라우트 유지)

```
/properties/:id/contract              → 계약 페이지
/properties/:id/contract/print/:formId → 양식 인쇄 페이지 (새 탭)
/customers                             → 고객 목록
/customers/new                         → 새 고객
/customers/:id                         → 고객 수정
/calendar                              → 일정 캘린더
```

### 7.2 계약 페이지 (`/properties/:id/contract`)

원본 `ContractClient` + `ContractPage` 를 SPA용으로 이식.

**구성**:
1. **상단 경고** — "실무 보조용 체크리스트·양식입니다. 공식·법적 효력을 보장하지 않습니다."
2. **매물 헤더** — 매물명·단지명·유형·거래유형, 목록으로 버튼.
3. **미설정 안내** — `realEstateType`·`tradeType` 없으면 "먼저 매물유형·거래유형을 설정하세요" + 수정 폼 링크.
4. **진행률 바** — `done/total`, 프로그레스 바(원본 스타일 유지).
5. **① 핵심 계약 데이터** — `requiredFieldKeys(tradeType)` 에 해당하는 필드들(라벨·값·수정 가능). 금액은 만원 표시, 날짜는 date input. `readOnly = status === "계약완료"`. 변경 시 `PATCH /properties/{id}` 호출(낙관적 업데이트).
6. **② 서류 체크리스트** — `resolveChecklist(g, t)` 결과, `PARTIES` 그룹별(공통/매도인/매수인/임대인/임차인/중개). 체크박스 토글 → `PATCH /contract/{id}/checklist/{itemId}`. 전체 체크/해제 버튼 → `POST /contract/{id}/checklist/all`. `required` 뱃지, `kind` 뱃지(서류/처리/신고/세금). 읽기전용 시 체크박스 비활성.
7. **③ 양식 인쇄** — `formsFor(g, t)` 버튼들. 클릭 시 `/properties/{id}/contract/print/{formId}` 새 탭(`target="_blank"`).
8. **하단 액션** — `status === "진행"` 이면 "계약진행 전환" 버튼(`POST /contract/{id}/start`). `status === "계약진행"` 이고 `progress.complete` 면 "계약 완료 처리" 버튼(`POST /contract/{id}/complete`). 완료 조건 미충족 시 비활성 + 안내 문구.

**상태 관리**: TanStack Query 키 `["contract", propertyId]`. 체크/필드 변경 시 낙관적 업데이트 + 서버 동기화.

### 7.3 양식 인쇄 페이지 (`/properties/:id/contract/print/:formId`)

- 서버 `GET /api/v1/forms/{propertyId}/{formId}` 가 반환한 HTML을 그대로 렌더(`dangerouslySetInnerHTML` 또는 `<iframe srcdoc={html}>`).
- 마운트 시 `window.print()` 호출 → 브라우저 인쇄 대화상자.
- A4 `@page` 마진·폰트(serif)·크기(210mm)는 서버 HTML에 포함.
- "실무 보조용 서식 — 공식 표준양식 아님" 문구 포함.

### 7.4 고객 페이지 (`/customers`, `/customers/new`, `/customers/:id`)

원본 `CustomerList`, `CustomerForm`, `page.tsx` 이식.

**목록**(`/customers`):
- 검색 input(이름·전화), 유형 Select(ALL + `CUSTOMER_TYPES`).
- 표(lg↑) / 카드(lg↓) 반응형. 컬럼: 이름, 전화, 유형 배지, 주소, 연결 매물(배지: 역할·매물명), 수정일, 관리(수정/삭제).
- 빈 상태: "‘새 고객’ 또는 매물에서 ‘고객으로 등록’으로 추가하세요."
- 삭제 확인 다이얼로그(`ConfirmDialog`).

**등록/수정 폼**(`/customers/new`, `/customers/:id`):
- 필드: 고객유형(칩 토글, 단순방문 단독/나머지 복수), 이름(필수), 전화(자동 하이픈 `formatTel`), 이메일, 성별(라디오: 남/여/미지정), 우편번호 검색(`PostcodeSearch`), 주소, 메모.
- 매물에서 열린 경우(쿼리 `?propertyId=&propertyLabel=`) 상단에 "출처 매물: {label}" 표시, 저장 시 `propertyId` 전송.
- 기존 고객이면 연결 당사자 배지 표시.

### 7.5 당사자 섹션 (매물 수정 폼 내)

원본 `property-form.tsx` 에 있던 "당사자" 섹션을 별도 컴포넌트(`PartySection`)로 분리해 매물 수정 폼에 포함.

**기능**:
- 현재 당사자 목록 표시(역할·이름·전화, 삭제 버튼).
- "당사자 추가" 버튼 → 다이얼로그: 역할 Select(`PARTY_TYPES`), 이름(필수), 전화.
- 저장 시 `POST /properties/{id}/parties` → 성공하면 목록 갱신, 고객 목록도 무효화(`["customers"]`).

### 7.6 일정 페이지 (`/calendar`)

원본 `CalendarView` 완전 이식. 핵심 포인트:

- **월 그리드** — `buildMonthGrid(year, month)` 로 6주×7일. 당월/전월/다음달 시각 구분.
- **사이드바(좌/우 반응형)** — 일정 유형 체크박스(미팅/임장/계약/기타 + 매물 일정). 선택일 패널(해당 날짜의 수기 일정 + 매물 파생 일정).
- **수기 일정** — 점 색상(`categoryStyle`), 클릭 시 상세 다이얼로그(`EventDetailDialog`).
- **매물 파생 일정** — 빨간 점(`PROPERTY_EVENT_STYLE`), 클릭 시 해당 매물 수정 폼으로 이동.
- **일정 추가** — 다이얼로그(`EventDialog`): 제목(필수), 날짜(기본 선택일/오늘), 시작시간(HH:MM, 선택), 카테고리(라디오), 메모, 매물 선택(옵션), 고객 선택(옵션).
- **상세→수정 전환** — 상세 다이얼로그에서 "수정" 버튼으로 편집 다이얼로그로 전환.
- **URL 동기화** — `?y=2026&m=10&d=20261015` 로 연·월·선택일 유지. 뒤로가기 시 복원.
- **로딩 상태** — 월 이동 시 "불러오는 중…" 표시.

**데이터 키**: `["events", year, month]`. 변경 시 무효화.

### 7.7 네비게이션·사이드바 확장 (`web/src/lib/nav.ts`)

1단계의 `navGroups` 에 아래 그룹 추가:

```ts
{
  label: "계약 관리",
  items: [
    { title: "계약 진행", href: "/properties/progress", icon: CircleDashed },
    { title: "계약 완료", href: "/properties/contracted", icon: CircleCheck },
  ],
},
{
  label: "고객 관리",
  items: [
    { title: "고객 목록", href: "/customers", icon: Users },
  ],
},
{
  label: "일정",
  items: [
    { title: "일정 관리", href: "/calendar", icon: Calendar },
  ],
},
```

⌘K 팔레트(`allNavItems`)도 자동 포함.

---

## 8. DB 스키마 참고 (이미 존재, 변경 금지)

| 테이블 | 핵심 컬럼 | 비고 |
|---|---|---|
| `"Property"` | `contractChecklist Json?`, `status`, `customerName`, `customerPhone`, `partnerName`, `partnerPhone`, `partnerManager`, `manager`, 날짜 필드 7개, `dealAmount`, `contractChecklist` | 체크리스트는 `Record<string, string>` (itemId → ISO timestamp) |
| `"Customer"` | `types String[]`, `name`, `zipcode`, `phone`, `address`, `email`, `gender`, `memo`, `propertyId?` | `types` 기본 `["단순방문"]` |
| `"PropertyParty"` | `propertyId`, `customerId`, `role` | `@@unique([propertyId, customerId, role])` |
| `"Event"` | `title`, `date(YYYYMMDD)`, `startTime?`, `category`, `memo`, `propertyId?`, `customerId?` | `@@index([userId, date])` |

새 서버는 **이 스키마를 읽고만 쓴다**. 마이그레이션 금지.

---

## 9. 검증 기준 (1단계 §5.1 확장)

1. 기존 운영 계정으로 로그인 → 계약·고객·일정 메뉴 진입.
2. 세 플랫폼 모두에서:
   - 매물 수정 폼에서 당사자 추가/삭제 → 고객 목록에 자동 반영.
   - 계약 페이지: 체크리스트 토글, 진행률 갱신, 계약진행→계약완료 전환, 필수 필드 미충족 시 완료 버튼 비활성.
   - 양식 인쇄: 6종 양식 버튼 클릭 → 새 탭 A4 렌더 → 인쇄 대화상자 → 데이터 자동 기입 확인.
   - 고객 목록: 검색·유형 필터, 등록·수정·삭제, 우편번호 검색.
   - 일정: 월 이동, 수기 일정 추가/수정/삭제, 카테고리 필터, 매물 파생 일정 표시, 매물 클릭→수정 폼 이동.
3. 새 앱에서 등록·수정한 데이터가 기존 웹(`resm.approid.team`)에 그대로 보이고, 반대도 성립.
4. 화면 디자인이 현재 웹과 동일(라이트/다크, 반응형, 컴포넌트 스타일).
5. 폰 폭에서 고객·일정 화면도 카드/리스트로 정상 표시.

---

## 10. 알려진 위험

- **양식 인쇄 HTML 렌더링**: 서버 사이드 템플릿이 원본 React 컴포넌트(`contract-document.tsx`)와 픽셀 단위 일치해야 함. CSS(`@page`, `mm` 단위, serif 폰트, 테이블 border-collapse) 검증 필요. 초기에 PDF 생성(`printcss`/`headless_chrome`) 대안도 고려.
- **체크리스트·필수 필드 로직 중복**: 화면(TS)·서버(Rust) 양쪽에 같은 로직이 들어가므로 `contract-checklist.ts` 를 **단일 소스**로 두고 서버는 포팅 테스트로 검증.
- **당사자 자동 매칭**: 이름+전화(숫자만)로 고객 중복 판정. 원본과 동일 로직(`custKey = name|phone_digits`) 포팅 필수.
- **일정 파생 동기화**: 서버가 월간 조회 시 매물 날짜 필드에서 파생 생성. 클라이언트도 같은 함수 쓰지만 서버가 권위. 월 경계 조건(전월/다음달 일정 포함) 검증.
- **Android 웹뷰 인쇄**: `window.print()` 가 웹뷰에서 동작하는지 확인 필요(안 되면 "PDF 저장" 버튼으로 대체).

---

## 11. 구현자에게 맡기는 것

- 서버 템플릿 엔진 선택(`tera`, `askama`, `handlebars`, `minijinja` 등). HTML 출력만 맞으면 됨.
- sqlx 쿼리 작성 방식(동적 UPDATE는 `QueryBuilder`).
- `contractChecklist` JSONB 파싱·직렬화 방식(serde_json::Value + 수동 매핑 vs 강타입 구조체).
- 통합 테스트용 테스트 데이터 시드 헬퍼.
- Tauri 앱에서 `window.print()` 동작 여부 확인 후 대안 결정.

---

## 12. 승인 대기

이 문서 검토 후 **[승인/수정 요청]** 을 주면 Task별 상세 계획(`docs/superpowers/plans/2026-11-15-resm-rust-stage-2.md`)을 작성해 구현에 들어간다.