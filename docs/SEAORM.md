# Persistence conventions

Purrsonnel uses SeaORM 1.1 over PostgreSQL. Application state holds a `DatabaseConnection`; services begin a `DatabaseTransaction`, call repositories, append audit evidence, and commit. SQLx is no longer called directly from application code. Its direct dev dependency supplies the existing isolated-database test harness; the same database pool is wrapped by SeaORM before requests enter the API.

## Boundaries

- `entities/`: 42 table models plus the read-only people-directory view, including nullable/composite primary keys and foreign-key relation definitions. Do not serialize a user entity directly to an API response: it contains private fields.
- `repositories/`: ActiveModel writes, entity selectors, typed identifier query builders, and named persistence operations. Filters and pagination are bound values; identifiers come from entity columns or explicit allowlists.
- `features/` and `auth/`: permission checks, transitions, validation and transaction orchestration. No SQL string assembly or direct driver calls.
- Private `postgres` modules inside repositories: reviewed PostgreSQL projections, multi-table/reporting queries, recursive policies, lease/locking statements, and conflict workflows that have not been expressed with entity builders. These remain parameterized SeaORM statements, not a claim that all hand-written SQL disappeared.

The directory uses entity queries with scoped conditions, subqueries, sort allowlists and separate total counting, including on empty pages. Audit inserts use ActiveModels and preserve database-supplied IDs/timestamps. Standard table operations use entity-column query builders. Query-result access is fallible; errors do not log private bind values.

Extend entities and repositories together. Prefer entity/ActiveModel APIs for ordinary CRUD. Keep transaction ownership in the workflow so a request, audience, inbox entry, outbox job and audit event cannot commit independently. Use PostgreSQL statements only for a concrete capability or a deliberately retained projection, and add regression coverage when changing them. Forward schema migrations and generated models must agree; this pass did not introspect a running database.

## Migration ownership

`src/migrations.rs` registers the five existing reviewed SQL migrations with SeaORM. PostgreSQL DDL, partial indexes, triggers and policy functions remain explicit migration inputs. Startup serializes migrations with a database advisory transaction lock. Migration execution and its registry are transactional. Seed installation defines a fictional seed function; only the development-auth startup path invokes it, under its own guard.

These migrations are forward-only because dropping tables would destroy immutable evidence. Add a new migration and model/repository changes for future schema work. Recover mistakes through a reviewed forward fix or a tested restore into a separate database, not automatic destructive rollbacks.

## Existing SQLx-managed databases

Fresh release Compose uses a separate volume, so there is no automatic overwrite of the earlier development database. Startup detects `_sqlx_migrations` and stops with an actionable error. It does not delete that table, mark SeaORM migrations as applied, replay schema creation, or infer that old evidence is safe to discard.

If an older database contains only disposable fictional data, keep it untouched and start the new release stack with its fresh volume. If it contains records worth retaining, back it up and rehearse transfer on a copy: verify each old migration version/checksum against the exact schema, inspect constraints/triggers/policies, and explicitly reconcile the SeaORM registry under a reviewed migration plan. No automatic baseline/adoption command is supplied. Do not delete the old migration table as a workaround.

Authored regression tests cover fresh SeaORM migration application/idempotency, legacy-history refusal without data loss, database-default ActiveModels and rollback, directory parameter binding/applicability, and API error redaction. Existing endpoint workflows exercise the SeaORM connection through the adapted PostgreSQL harness. Runtime execution is still pending.
