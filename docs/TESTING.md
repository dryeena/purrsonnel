# Testing strategy

Frontend ESLint, Prettier, TypeScript, 78 tests across 10 files, and the Vite build now pass on the coding machine using project-contained tools. Rust/PostgreSQL suites, Docker startup and browser/device acceptance remain unexecuted. The frontend result is not a claim of a verified full-stack release.

## Automated layers

1. `scripts/verify-source.ps1`: dependency-free checks for required files, JSON parsing, exact brand asset hashes, SVG parsing, relative TypeScript imports, sensitive browser patterns, core database tables/triggers/indexes, route uniqueness, and minimum authored test coverage counts. This cannot parse or compile Rust, TypeScript, or SQL.
2. Rust unit tests in `apps/api/tests/domain.rs`: fail-closed environment/origin configuration, cryptographic token behavior, bounded text and years, rejection reasons, nested whitelist filters and complexity bounds, bulk operation escalation, controlled schema/answer types, invalid URL/date handling, and exclusive/bounded duration input modes.
3. PostgreSQL integration tests in `apps/api/tests/workflows.rs`: SQLx creates disposable fixture databases; the application uses a SeaORM connection around that pool. Real Axum requests via Tower cover cookies, CSRF, origin/host validation, session rotation, scoped grants, memberships, graph cycles, explicit service years, separate term signatures, republishing/evidence immutability, volunteer QR decision/replay, self/scope rejection, expiry/rotation, atomic owner hiring, nonfinal panel responses, note/answer privacy, snapshot conflicts, bulk confirmation/audit, and dedicated permission grant authorization.
4. Frontend Vitest + Testing Library tests: bootstrap/CSRF transport, safe error propagation, permission hints, spreadsheet injection escaping, form types/errors/pending state, dialog semantics/focus/Escape, explicit mutation failure, status labels, and dashboard HR/member priorities with API fixtures.
5. PowerShell smoke checks: development auto-login and authenticated reads through the actual Vite/API proxy.

Tests do not require installing a browser. The test Compose file and CI workflow describe the complete runtime commands. SQLx's test role must be permitted to create isolated test databases. Use disposable data only.

Second-pass tests add request scope/private inboxes, team recipient snapshots, idempotent sends, concurrent claims, real outbox creation with a fictional key fixture (no network), disabled push configuration, endpoint SSRF boundaries, subscription key validation, public-only worker caching, generic push content/vibration options/safe click URLs, explicit installation gestures, dirty dialog protection, offline form editing, and stable background refresh. Service-worker tests run its actual JavaScript in an isolated Node VM with mocked platform I/O. They do not claim actual browser/OS delivery; use the physical-device checklist in HOME_SCREEN_AND_REQUESTS.md.

The polish pass adds filter-before-pagination/privacy and literal-search tests in PostgreSQL; frontend tests cover overdue rules, relative timestamps, progress/cancellation semantics, field descriptions/character counts, error focus, server-success-only feedback, and status-filter interaction. Device-enabled status is checked for the current owner, and opt-out is asserted in the configured-outbox workflow. No visual snapshot claims are made.

## Manual acceptance on the real machine

- Clean Compose boot: migrations + fictional seed + frontend boot without manual DB repair.
- Inspect cookie flags; reload preserves identity; switch rotates session and loses previous privileges.
- HR overview, directory filters/pagination, exact nonconsecutive year counts, memberships and primary department, skill assignment, and inverse directs.
- Lead cannot read another department; director cannot use settings/audit; volunteer cannot invoke HR endpoints directly.
- Two applicable volunteer policies, typed affirmation, independent signatures, new published version requiring a fresh signature, prior evidence retained.
- Volunteer logs 90 minutes, submits, rotates code, shares it; staff previews without mutation and explicitly approves. Replay, expiry, self-approval, missing reason, and wrong department fail.
- Staff shortcut code lands on `/volunteer` and never acts as an approval reference.
- Camera grant/denial/unsupported browser; link fallback always works; camera turns off when dialog closes.
- Post a position with default + custom questions, including multi-select; save incomplete application, resume, submit. Ordinary staff may apply but cannot create postings.
- Schedule panel and timezone; record separate recommendations; notes respect audiences; owner/HR override finalizes once; Other mapping is explicit.
- Saved-list AND/OR filters, review snapshot, confirmation, execution outcomes, conflict after concurrent edit, downloadable results, per-person audit.
- System identity grants/revokes scoped permissions with audit; last system admin protection.
- At 375px and tablet/desktop widths: sidebar, tables, forms, QR modal, and long text remain usable. Keyboard-only navigation, modal focus, focus restoration, screen reader names, reduced motion, and contrast review.

## Remaining test work

`apps/api/tests/seaorm.rs` adds ORM query-rendering/scope and error-redaction unit cases plus fresh migration/idempotency, database-default ActiveModel rollback, and legacy SQLx-history refusal tests. The Docker release CI job is authored to build the actual runtime images, wait for health, and probe the static SPA and fictional login through nginx; it has not been observed here. The inventory is 26 Rust unit cases, 54 PostgreSQL cases, and 78 expanded frontend cases. Only the frontend cases have executed.

The frontend readability pass also covers the final hiring dialog's recommendation/explicit-decision flow and the modal's latest-close-callback behavior. The existing required-picker test exposed and now verifies the fix for a browser silently selecting the first real option. See FRONTEND.md for linting, formatting and file-creation conventions.

The follow-through pass adds authored cases for private/validated preferences; overnight/daytime/DST quiet-hour policy; inbox preservation and no replay after opt-in; deferred jobs without consumed attempts; pending-policy changes; sender-only scheduling and reminder cooldown; assignee-only reminders; competing scheduler cycles; closed-work skips; revoked-scope escalation; audience expansion only to an explicit target; immutable activity; cancellation updates; clock input/schedule serialization; and preference-save failure recovery. Worker payload tests cover generic reminder, escalation, and update messages. These cases are authored, not executed on the coding machine.

Run and repair the authored suites before sign-off. Concurrent QR/finalize race tests and pending-identity provisioning tests are authored. Add verified external-link ownership tests when the provider exists, timezone/DST edge cases, production TLS/proxy tests, browser automation and accessibility scanning once that infrastructure is available. The repository does not claim these checks have passed.
