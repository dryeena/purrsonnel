# v0.1.0 development release

The initial release is prepared in source for a Docker-capable machine. It is not published or certified. Its intended audience is the project team using fictional development data on a local/private machine. It includes release-style packaging, not production authentication.

## Prepared

- Application manifests identify version 0.1.0.
- Runtime persistence uses SeaORM; entities/repositories are separate from workflows.
- Forward migrations retain the existing schema, evidence guards and policy semantics.
- Docker release images build the Rust binary and static frontend; nginx serves the SPA/API on one origin.
- Compose isolates the database network, persists a dedicated volume, probes health, and keeps fictional login on loopback.
- Documentation describes setup, configuration, operational boundaries, test commands and remaining scope.

## On the destination machine, before tagging

1. Read BUILD_PROGRESS.md and SEAORM.md. Back up any existing database before changing its migration owner.
2. Use the checked-in frontend lockfile with `npm ci`. Generate and review the still-missing backend `Cargo.lock`. Pin image digests for a reproducible distributable.
3. Run the API, PostgreSQL and frontend suites with the isolated test Compose file; fix findings. Run Rust formatting and frontend typechecking/build.
4. Start the release-style stack on a fresh dedicated volume. Confirm migration idempotency, fictional login, scoped workflows, health checks and shutdown.
5. Review mobile/keyboard behavior, installation and real-device delivery separately when the appropriate authenticated host is available.
6. Review the README/changelog against the actual observed result. Only then create a Git tag and GitHub release if authorized.

Production additionally requires the real authentication integration, deployment secrets, appropriate database roles, reviewed backup/restore and retention operations, HTTPS, and completed policy decisions. Do not weaken the development-origin guard to publish this preview.
