# Move Purrsonnel to the development machine

This repository contains the application source, supplied branding, authored tests,
database migrations, and development containers. No production credentials or live
personnel records are needed for the fictional development environment.

1. Clone the repository on the destination machine.
2. Read `docs/BUILD_PROGRESS.md` for implemented features and remaining gaps.
3. With Docker and Compose available there, copy `.env.example` to `.env`.
4. Run `docker compose -f docker-compose.release.yml up --build -d` from the repository root.
5. Open `http://localhost:8080`. Development authentication uses fictional accounts.

For host-native development and test commands, see `README.md`, `docs/FRONTEND.md`
and `docs/TESTING.md`. Frontend lint, formatting, TypeScript, 78 tests and the Vite
build pass locally. Its dependency lockfile is checked in. The Rust/backend
suites, Docker image startup, database migrations and browser/phone acceptance
remain pending. Generate and review `Cargo.lock` on the destination machine.
Source checks and frontend success do not certify the full stack. No push
credentials or live phone alerts were created here.
Real authentication and deployment remain separate integration work. Phone alerts
are disabled by default. Keep secrets in the destination machine's ignored `.env`
or its deployment secret store. Do not commit credentials, database backups, or
portable tool downloads. Do not expose the development-auth environment publicly.

The release-style stack serves built assets through nginx and uses a dedicated database volume. Read [SeaORM conventions](SEAORM.md) before transferring any legacy database. For the Vite development stack, use the separate commands in README.md.
