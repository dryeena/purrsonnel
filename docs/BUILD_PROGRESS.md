# Build progress — 2026-11-09

User constraint: **code only on this machine; install nothing**. No package manager, Rust installer, Docker build, server, or external service was run. No hosting registration or publication was attempted.

## Verification status

- `Get-Command node,cargo,rustc,docker,npm,npx`: no installed toolchain commands found. Checked conventional Node/Cargo/Docker locations as well; no executable found. Git Bash exists but does not supply these toolchains.
- Read the specification in UTF-8 and inspected both brand assets.
- Created source, configuration, migrations, fictional fixtures, API/UI flows, tests, CI, and handoff documents.
- Source verification command: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-source.ps1`. See the appended verification result below.
- **Not run:** `cargo test`, Rust compiler/formatter, PostgreSQL migrations, Docker Compose, frontend dependency resolution/typecheck/build/Vitest, browser preview, camera scanning, or smoke HTTP tests. They require the other machine. No milestone is certified complete until those pass.

## Milestone A — foundation

- [x] Rust/Axum/SeaORM source and feature router (SeaORM replaces the original direct SQLx layer); PostgreSQL schema; Preact/TypeScript/Vite/Tailwind source; Docker/Compose; environment template.
- [x] Dev bootstrap, session reuse/rotation, principal extraction, Origin/Host/CSRF checks, mutation limiting, fail-closed startup, fictional identities, visible identity switcher.
- [x] Health endpoints and Utoipa health schema, safe errors, relative frontend proxy, README/native commands, full feature API documentation.
- [ ] Execute clean Compose boot, migrations, auth tests, frontend build and real API fetch.

## Milestone B — people and organization

- [x] Scoped directory/search/filter/sort/pagination; details/edit; multiple memberships/primary; managers/directs and cycle guard; skill assignment; explicit verified service years; CSV escaping; audit.
- [x] Division/department outline and creation, separate levels, typed custom fields and values, dedicated permission grant/revoke APIs and system UI.
- [ ] Run integration/UI tests and responsive browser QA.
- [x] Department edit/lead UI, division edit API, custom-field visibility/self-edit/length rules, pending-profile intake, and audited administrator record export.
- [ ] Complete division editing UI, full skill catalog CRUD, richer numeric/date custom-field rules, and a comprehensive subject-access export that reconciles all retained data and privacy obligations.

## Milestone C — terms and volunteers

- [x] Separate applicable documents, draft/new version/publish, immutable versions and signature evidence, typed affirmative acknowledgment/receipt, compliance read.
- [x] Time drafts/submission/withdrawal, duration or timezone-aware start/end, overlap guard, approval QR issuance/rotation/expiry, authorized preview/explicit decision/self-rule/idempotency, persisted approving actor, reports.
- [x] Distinct navigation QR, local QR generation/copy fallback; optional native camera detector with manual fallback.
- [ ] Run DB/HTTP/frontend tests and scan real QR codes on target browsers.
- [x] Draft time editing, supervisor reopening with immutable original snapshots/decision history, configurable duration cap, current-identity eligibility recheck at submission, visible polling, and token-bound decision idempotency.
- [ ] Add configurable event-specific eligibility policies, retention cleanup, and rich sanitized Markdown presentation.

## Milestone D — hiring

- [x] Position creation/open/close, explicit owner/level/leadership, snapshotted organization defaults + controlled role questions, reordering, applicant drafts/edit/submit, review/redaction.
- [x] Immutable template/version APIs, interview scheduling/panel validation, notes visibility, separate recommendations, explicit owner/HR finalization transaction/idempotency, membership update, pending identity preservation, notifications.
- [x] System UI maps Other before hiring; rank is separate from permissions.
- [x] Template-library/version UI and posting reuse, interview rescheduling/cancellation/panel additions, audited hiring-owner reassignment, and authorized intake that pre-provisions unlinked profiles/applications. Pending identities cannot be activated through a normal personnel edit.
- [ ] Run tests; finish interviewer removal/reassignment history and richer posting-edit fields.

## Milestone E — HR scale and polish

- [x] Typed bounded nested filter evaluator; UI AND/OR criteria; private saved lists; list results; reviewed bulk snapshots; explicit confirmation; conflict reporting; narrow safe operations; results CSV; per-person audit; paged audit UI.
- [x] Responsive plum design, supplied assets, native forms/dialog focus, clear error/loading/empty states, member and HR dashboards; Rust unit/DB workflow tests and frontend component/API tests authored; CI and isolated test Compose.
- [ ] Run all tests and fix findings; format with toolchains and generate dependency lockfiles.
- [x] Saved-list edit/HR sharing UI and API, own notification bell/read acknowledgment, and record-detail API routes.
- [ ] Extend nested-group visual builder, SQL AST compilation above 1,000 scoped people, additional safe bulk operation types beyond status/eligibility, permission-scoped global search beyond people, and browser/accessibility automation.

## Sign-off

This is a substantial implementation prepared for the development machine, **not a verified first release or completed specification**. The checkboxes above distinguish written paths from required runtime validation and remaining scope. Future myFWA OIDC, legal approval, actual event policy, and email transport remain separate integrations as requested by the spec.

## Verification log

1. First source-check invocation failed because `docs/API.md` had not yet been written. Added the endpoint contract and reran successfully.
2. Subsequent source passes checked imports, copied-asset integrity, schema invariant declarations, Rust module paths, route uniqueness, and SQL placeholder/bind counts. These are static checks, not SQL execution or typechecking.
3. Authored 17 Rust unit tests, 31 isolated PostgreSQL workflow tests (including concurrent approval/finalization), and 23 frontend test declarations (29 expanded cases, including parameterized CSV-injection inputs).
4. `git status --short` reported that the starting folder is not a Git repository. Source files are present on disk; no commits were created.
5. Final source-check result is recorded below after the final pass. Runtime commands remain pending exactly as listed above.

```text
PASS: 343 source/asset checks. 70 routes; 17 Rust unit tests; 31 PostgreSQL workflow tests authored.
Frontend: 23 test declarations / 29 expanded test cases across 4 test files.
Total authored runtime cases: 77. Runtime cases executed here: 0.
```

The source checker itself passed on this machine. It is intentionally not reported as a successful application build or as passing migrations/runtime tests.

## Second pass — home screen, usability, and requests

User asked for an easier mobile experience, home-screen installation, and requests that alert a team's or individual's phone.

- [x] Web app manifest with stable identity, standalone mode, app shortcuts, regular/maskable PNGs, Apple touch icon, and safe-area metadata. PNG assets were exported from the original SVG using Windows-native drawing APIs; original asset hashes are unchanged.
- [x] Native install prompt when available and browser-specific fallback instructions; installed-mode detection; public offline help; opt-in app updates; no private runtime/API caching or offline mutation queue.
- [x] Mobile Home/Requests/Volunteer/Me navigation, usable touch targets, focus-managed sidebar, unique dialog labels, unsaved-form dismissal/navigation warnings, offline save prevention with editable in-memory input, saved confirmation, and background polling without spinner replacement.
- [x] Requests feature/API: scoped recipient catalog, team/individual compose, server-paginated inbox/sent views, detail links, team recipient snapshots, atomic claiming/completion/cancellation, audited transitions, and exact-payload send idempotency.
- [x] Own notification bell links to requests, persisted update notifications, dashboard incoming-request count, and phone settings that report unconfigured/unsupported/blocked states accurately.
- [x] Browser permission/subscription and opt-out handling; identity-switch detachment; server-side VAPID configuration validation; AES128GCM/VAPID Web Push delivery; bounded HTTP client/host allowlist; PostgreSQL outbox leasing/retries/expiry; generic lock-screen notification and safe deep links. No push keys were generated or alerts sent here.
- [x] Native asset export and source checks executed successfully. Added tests for request authorization/idempotency/claim races/outbox creation, push endpoint/key validation, worker privacy/offline/click behavior, install prompts, form loss prevention, offline edits, and background refresh behavior.
- [ ] Execute the Rust/DB/frontend suites and clean Compose build on the real machine. This machine still has no Node/Cargo/Docker, and no dependency installation was attempted.
- [ ] Perform real Android/iPhone/desktop install and background/locked-screen delivery checks. Phone alerts require configured server keys, a suitable secure authenticated host, device subscription, and OS permission/settings. Development auth remains restricted to loopback and production remains fail closed.

Second-pass checks: `scripts/build-icons.ps1` exported all four requested PNG sizes. `scripts/verify-source.ps1` passed **430 checks**, including manifest/icon dimensions, public worker boundaries, SQL bind counts, imports, and original-brand integrity. All PowerShell scripts parse. Runtime tests remain **unexecuted**, with **20 Rust unit tests, 37 PostgreSQL workflow tests, and 43 frontend declarations (55 expanded cases)** now authored. See [home-screen and request setup](HOME_SCREEN_AND_REQUESTS.md).

## Final polish pass

- Added a separate, local `polish.css` layer: readable dashboard/table/helper typography, consistent card depth/radii, restrained interaction transitions, loading placeholders, long-text handling, sticky dialog headings, mobile spacing, and reduced-motion support. No fonts, packages, or new artwork were downloaded.
- Forms now separate accessible names from descriptions, expose associated help text and live character counts, clarify draft saves, focus failed-submission messages, and show consistent busy/saved states. Native field validation and server validation remain authoritative in their respective roles.
- Added brief, dismissible success feedback for request transitions, announced only after a successful API response. Timers pause for pointer/focus reading. Errors keep their existing visible, announced feedback.
- Requests now support scoped server-side search/status filtering before pagination, explicit refresh/retry, clear-filter actions, useful empty states, relative timestamps with exact-date tooltips, overdue indicators for unfinished work, and status-based progress without invented timestamps.
- Refined request-send receipts and phone-alert copy. Device “enabled” status is checked against the authenticated server record, not just a browser subscription that may have become stale. Opt-out/owner isolation remains enforced and is covered in the configured-outbox integration scenario.
- Fixed active mobile navigation for query-based profile links and excluded disabled fieldset controls from modal focus cycling. Existing home-screen/offline/privacy boundaries are unchanged.
- Added regression tests for filter visibility/pagination, literal search wildcards, overdue/relative-time behavior, progress semantics, accessible form hints/counters, focused errors, and success-only notifications. Cosmetic changes do not have brittle snapshot tests.

Final source pass: **443 source/asset checks pass; 77 API paths, 20 Rust unit tests, 39 PostgreSQL workflow tests, and 52 frontend test declarations (64 expanded cases) are authored.** Runtime cases remain unexecuted because the coding machine has no toolchain. Visual/mobile/browser/device acceptance remains for the real machine. Nothing was installed, deployed, or sent to another person.

## Feature strengthening — requests and notification policy

User deferred validation and asked to shore up features. This pass implements local source work:

- Sender-controlled manual reminders with a 15-minute cooldown; accepted requests remind only the active assignee. Empty/inactive audiences produce an actionable error or a skipped event instead of false delivery success.
- One-shot scheduled reminders and explicit-person escalation, with schedule replacement/removal. Relative delays use the server clock. Escalation adds only the selected person when still unclaimed, rechecks active accounts and current sending scope, and records its authorization and outcome.
- An independent database scheduler, transactional inbox/outbox/history writes, competing-worker row locks, and durable processed markers. Closing work stops notifications. Scheduler steps survive process restarts and also function with push unconfigured.
- Account-wide all/urgent-only/inbox-only phone policy and daily quiet hours in a validated IANA timezone. Quiet-hour waiting consumes no transport attempts; enqueue and dispatch apply the same PostgreSQL policy. Pending jobs reevaluate changed preferences; previously suppressed jobs do not replay after opt-in.
- Sender-only aggregate delivery summaries and immutable participant timelines, with scheduled times separate from actual events and no phone-read/buzz claim. Notification clicks retain the existing safe deep-link boundary. Reminder, escalation, and update lock-screen messages stay generic.
- Twenty-four-hour outbox expiry, stale-alert suppression, lease tokens to protect newer worker results, bounded logging without private payloads, due-step indexes, and indexed request-notification lookup.
- Added regression cases covering scheduling, scoped access, concurrent workers, removed schedules, inactive recipients, quiet-hour/DST policy, notification preservation, preferences, and UI failure recovery. Updated the [feature guide](REQUEST_FOLLOW_THROUGH.md), API contract, architecture, and test inventory.

Source consistency: **484 source/asset checks pass; 80 API paths; 23 Rust unit tests, 51 PostgreSQL workflow tests, and 62 frontend declarations (76 expanded cases) authored.** Nothing was installed, launched, migrated, connected, or sent; runtime suites remain unexecuted. This is feature implementation with source consistency checks, not application/device validation or release sign-off.

## SeaORM migration and v0.1.0 release preparation

- Replaced direct SQLx application access with a SeaORM connection, transactions, errors, and migration registry. SQLx remains a dev dependency for disposable database fixtures and a transitive SeaORM driver.
- Added 42 entity models plus the read-only directory view and feature persistence repositories. Ordinary persistence uses ActiveModels/entity selectors/entity-column SeaQuery builders. There are 157 builder operations and 96 retained PostgreSQL statement operations, plus the dedicated ORM directory and ActiveModel paths. Retained statements cover projections, policy/concurrency and conflict workflows and are explicitly isolated in private repository modules; this is not described as eliminating all SQL.
- Removed SQL string assembly and driver calls from auth/feature workflows. Directory filtering uses entity conditions/subqueries with bound text and a sort allowlist; total counts remain accurate on empty pages. Result extraction is fallible, and database errors omit private query values from logs.
- Registered the five existing immutable SQL inputs with SeaORM's forward migration history. Startup serializes migration execution and refuses legacy SQLx-managed databases instead of replaying DDL or deleting evidence. No database was migrated here.
- Adapted all existing API test state construction to wrap the fixture pool with SeaORM. Added query-rendering/privacy tests and isolated fresh-migration, idempotency, rollback/defaults, and legacy-history-refusal cases.
- Added a multi-stage static-web/nginx release image and `docker-compose.release.yml`: same-origin API proxy, private database/API, dedicated persistent volume, loopback web port 8080, health checks, API nonroot runtime, graceful shutdown and lease recovery. Development Compose explicitly selects the Vite stage. The test image includes the required native OpenSSL build dependencies.
- Rebuilt README with Docker quickstart, configuration, first-session guidance, features, architecture, extension points, tests, persistence and release limits. Added changelog, SeaORM conventions, release checklist and updated deployment/machine handoff. Auth remains development-only and production remains fail closed.
- Added a GitHub Actions job to build/start/probe/stop the packaged Docker stack after publication. No workflow was triggered or observed here; GitHub handoff remains unfinished.

Latest source consistency result: **1,537 checks pass; 80 API paths, 26 Rust unit tests, 54 PostgreSQL workflow tests and 62 frontend declarations (76 expanded cases) authored.** The 156 runtime cases, Docker image builds, migrations, formatting and browser/device acceptance remain unexecuted. Nothing was installed, downloaded as a runtime dependency, deployed, published or sent. This prepares an initial development release; it does not certify a production release. Dependency lockfiles and image digest pinning await the destination machine.

## Frontend readability, linting and first executed checks

The user explicitly authorized project-contained frontend tooling, provided it can be removed with the project folder. Portable Node, npm configuration/cache/logs and temporary files live under `.tools/`; dependencies live under `apps/web/node_modules/`. No global package installation or permanent environment change was made. Rust and Docker remain uninstalled/unexecuted here.

- Replaced compressed feature files with cohesive feature folders and public entry points. Split shared UI into forms, dialogs, actions, primitives and icons. Separated HTTP/auth services, reusable data hooks, types and pure helpers; retained a compatibility export for older API imports.
- Added a frontend creation guide and scoped AGENTS.md, ESLint 10 flat configuration, Prettier, EditorConfig, stable import ordering, type-only imports, hook/dependency rules, stable JSX keys, explicit button types, unsafe HTML/link guards, native-Preact import boundaries and a 400-line module guard.
- Formatted frontend source, styles, tests, public HTML/worker/manifest and configuration. Original branding bytes remain unchanged.
- The first real formatter/linter exposed malformed JSX in the final hiring dialog and the missing notes-dialog boundary. Fixed these, hook cleanup/callback issues, unstable/missing keys and the native required-select default. Added hiring-decision and latest-dialog-callback regression tests; the existing required-picker test now passes.
- Updated compatible lint/test dependencies, removed critical findings from the older test pool, locked the frontend dependency graph, and applied the documented patched minimatch override. npm reports **zero vulnerabilities** for the installed graph.
- Docker and web CI now use `npm ci` and enforce lint/format/type/test/build checks. Updated README, changelog, testing and machine handoff to distinguish actual frontend results from pending backend/full-stack work.

Executed successfully on this machine:

```text
ESLint: zero errors or warnings (max-warnings 0)
Prettier: all matched files formatted
TypeScript: tsc --noEmit passes
Vitest: 10 files, 78 tests pass
Vite: production frontend builds, 132 modules transformed
Source/asset consistency: 1,906 checks pass
Documentation: local links and text encoding checks pass
```

There are now 158 authored runtime cases: **78 frontend cases executed successfully; 26 Rust unit and 54 PostgreSQL cases remain unexecuted.** Docker/backend compilation and startup, database migrations, visual/browser/device checks, real identity integration and phone sends remain pending. No GitHub upload, release publication, real credential or live notification was performed during this pass. See [frontend conventions](FRONTEND.md).
