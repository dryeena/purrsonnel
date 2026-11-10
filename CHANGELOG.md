# Changelog

## 0.1.0 — Initial development release (prepared 2026-11-09)

### Included

- Personnel, organization, agreement, volunteer, hiring, saved-list, bulk-change, and audit workflows.
- Team/person requests with atomic claiming, reminders, explicit escalation, phone preferences, quiet hours, and activity history.
- Responsive UI using the supplied branding, home-screen installation, safe public offline help, form-loss guards, and accessible interaction states.
- SeaORM entity models and persistence repositories, entity-based directory filtering, transaction-based workflows, and a SeaORM migration registry.
- Multi-stage Docker images and a local release-style Compose stack with compiled assets/nginx, private PostgreSQL, persistent storage, readiness checks, and graceful API termination.
- Authored PostgreSQL, Rust, and frontend tests, source checks, CI, and machine handoff documentation.

### Limits

This is a development preview prepared in source, not a tested production release. Frontend lint, formatting, typechecking, 78 tests and the static build pass locally. Backend builds/tests, migrations, Docker startup and browser/phone checks remain pending on the Docker machine. Real myFWA authentication is unimplemented and production startup fails closed. The frontend dependency lockfile is checked in; backend lockfile generation and pinned image digests remain destination-machine release work. Full original-spec scope is tracked in `docs/BUILD_PROGRESS.md`.

SQLx is no longer used directly by application code. It remains a dev dependency for disposable PostgreSQL test fixtures and is also SeaORM's underlying driver. PostgreSQL-specific projections and concurrency/policy queries remain isolated in repositories; this migration does not claim database portability.

The release Compose stack uses a new named volume. Legacy databases tracked by SQLx are rejected rather than replaying schema migrations over existing personnel evidence. See `docs/SEAORM.md` before moving existing data.

### Frontend readability and linting

- Split pages, dialogs, shared UI controls, HTTP/auth services, the data hook, types and pure helpers into cohesive modules.
- Added ESLint 10, Prettier, import/type/hook/JSX rules, module-size and native-Preact import guards; wired checks into CI and Docker.
- Fixed malformed hiring JSX, an unreachable notes dialog, stale modal callback handling, form cleanup tokens and required-picker defaults exposed by the checks.
- Formatted frontend source, tests, styles and public files while preserving original branding. The installed dependency graph currently reports zero npm audit vulnerabilities.
