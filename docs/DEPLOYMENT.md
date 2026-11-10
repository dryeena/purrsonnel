# Development-machine handoff and deployment

## First run

For the initial release-style Docker package, follow the README: `docker compose -f docker-compose.release.yml up --build -d`, then open `http://localhost:8080`. This builds static web assets served by nginx, a release Rust binary, and private PostgreSQL. The root default Compose file remains the Vite development stack. Neither is configured for public authentication.

1. Copy the entire repository, including brand assets, migrations, tests, and dotfiles.
2. On the real development machine, use the README Compose commands. The frontend already passes local checks; the Docker/backend build and platform-specific dependencies still need verification there.
3. Run API tests against the isolated test database, frontend typecheck/tests/build, and smoke tests. Resolve failures before treating any milestone as complete.
4. Run `cargo fmt` and commit the formatted sources. The frontend lockfile is checked in; use `npm ci`. Generate and review the backend `Cargo.lock` there, then use `cargo --locked`. Frontend tooling was explicitly authorized locally; Rust remains uninstalled here.
5. Manually verify mobile, keyboard, QR camera fallback, and all seeded identities using the checklist in TESTING.md.

`MAX_VOLUNTEER_MINUTES` configures the per-entry cap (1–960, default 960). The server enforces it, and the UI receives it with organization metadata. Event-specific eligibility policies still need organization-approved configuration.

## Data

Development PostgreSQL persists in `postgres_data`; release-style Compose has its own `release_postgres_data` volume. SeaORM migrations run at API startup. Seed is guarded and does not rerun when demo users already exist. For schema changes, add a new numbered migration; do not rewrite an applied migration. Back up important data before any destructive operation.

Reset is deliberately manual: stop development services, inspect `docker volume ls`, identify the exact Compose project development volume, and remove only that volume after confirming it contains disposable data. Never run reset commands against the test or real production environment by assumption.

Test Compose uses a separate, ephemeral database. It does not mount the development database volume.

For a manual release-stack backup, create a database archive inside the container, then copy it out. This avoids binary redirection problems in Windows PowerShell:

```sh
docker compose -f docker-compose.release.yml exec -T db pg_dump -U purrsonnel -d purrsonnel -Fc --file=/tmp/purrsonnel-backup.dump
docker compose -f docker-compose.release.yml cp db:/tmp/purrsonnel-backup.dump ./purrsonnel-backup.dump
```

Keep the archive outside source control in an appropriate private backup location. A backup is not proven recoverable until restored into a separate database and checked. Stop writers during a full application/data transfer when consistent external effects matter. Do not restore over existing evidence as an unattended upgrade step. Legacy SQLx migration history requires the explicit procedure in SEAORM.md; startup refuses automatic schema replay.

## Production design

Production deployment is **blocked by the unimplemented myFWA adapter**, and startup enforces this. Do not bypass the guard to launch a public demo.

The intended single-server arrangement is a TLS reverse proxy serving `apps/web/dist`, forwarding same-origin `/api` to a loopback Rust service, plus private PostgreSQL. Build the frontend once and run the Rust binary as an unprivileged user. Maintain SPA fallback for non-API routes; return real errors for API paths. Use private credentials, least-privilege DB roles, reliable backups and restore tests, patching, observability, retention jobs, and HTTPS cookies. The default Compose web service is Vite; the release-style stack explicitly selects the static nginx runtime stage. Production integration work is still required.

Use separate migration-owner and runtime database roles before production. Restrict runtime `UPDATE/DELETE` on evidence and audit tables in addition to existing database triggers. Configure CSP such as `default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'` at the production proxy, review inline style usage, and narrow it where practical. Camera permissions remain optional.

No public deployment, external credentials, account configuration, or package installation was performed on the coding machine.
