# Frontend conventions

Read `../../docs/FRONTEND.md` before adding or reorganizing frontend code.

- Use Preact and `preact/hooks`; do not introduce React compatibility wrappers.
- Keep each feature under `src/features/<domain>/` with a small `index.ts` public entry point.
- Separate page orchestration, dialogs and cohesive interactive controls. Extract by responsibility, not every HTML element.
- Put shared HTTP transport in `src/services/`, reusable lifecycle/data hooks in `src/hooks/`, pure helpers in `src/lib/`, and explicit contracts in `src/types/` or the owning feature.
- Use descriptive kebab-case filenames, PascalCase components, named exports, and `import type` for types.
- Keep JSX declarative, callbacks named when substantial, hooks unconditional, lists keyed by stable identity, and button types explicit.
- Prefer named DTOs and `unknown` for untrusted data. `Row` is a documented legacy boundary for existing dynamic API contracts; do not spread new `any` annotations through features.
- Keep modules below the configured 400 nonblank/noncomment lines. Format with the project Prettier config; do not hand-compress statements or JSX.
- Preserve CSRF, same-origin credentials, permission checks, unsaved-form protection and public-only service-worker caching.
- After changes, run `npm run check` and `npm run build`. Fix lint findings rather than broad rule suppressions. Add behavior tests when fixing logic; avoid cosmetic snapshots.
- On this coding machine use `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/web.ps1 ...` from the repository root. Its tools, npm config/cache and temporary files stay inside the project. Do not install global tools or change persistent environment settings.
