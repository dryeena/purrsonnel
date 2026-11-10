# Readable Preact code

The frontend uses responsibility-based modules, ESLint for correctness and maintainability, Prettier for mechanical formatting, and TypeScript for contracts. Formatting should be automatic; meaningful names, component boundaries and data flow still require judgment.

## Where files belong

```text
apps/web/src/
  app.tsx                         Application shell and routing
  main.tsx                        Browser startup
  features/
    people/
      index.ts                    Public feature exports
      people-page.tsx             Directory state and page composition
      person-details.tsx          Profile dialog and its interactions
    hiring/
      interviews-page.tsx
      finalize-dialog.tsx
      notes-dialog.tsx
      application-fields.ts       Pure field-schema mapping
    requests/
      request-inbox.tsx
      compose-dialog.tsx
      request-details.tsx
      request-progress.tsx
  components/
    ui/
      form.tsx                    Shared form semantics
      modal.tsx                   Focus, dismissal and unsaved-state guards
      primitives.tsx              Small related visual controls
      action.tsx                  Async action feedback
      icon.tsx                    Existing vector icon vocabulary
      index.ts                    Shared UI exports
  services/
    api-client.ts                 Fetch, same-origin credentials, CSRF, errors
    auth-service.ts               Authentication request orchestration
  hooks/use-data.ts               Loading, polling and stale-request protection
  lib/                           Pure helpers and browser capabilities
  types/                         Shared API/browser contracts
```

Keep a feature-specific API adapter, hook or type beside its feature when only that feature uses it. Move a module into a shared folder only when it has a clear shared responsibility. Avoid generic repositories, controllers or global stores that obscure a simple Preact flow. `lib/api.ts` is a compatibility re-export for older imports; new source imports the owning service/hook/helper directly.

## Creating a module

1. Name the file after its role: `approval-scanner.tsx`, `notes-dialog.tsx`, `request-progress.tsx`. Use `.ts` when there is no JSX.
2. Prefer one substantial component per file. Keep small helpers together when they serve that component. A new file should reduce context switching, not hide a few obvious lines behind an abstraction.
3. Use named exports. Export only the feature's public surface through `index.ts`; sibling components import each other directly to avoid barrel cycles. Types use `import type`/`export type`.
4. Inside a component, put hooks first, then derived values, handlers, and JSX. Use descriptive names such as `interview`, `participant`, `selectedDepartment` and `saveRequest`.
5. Separate error/loading/empty/success states clearly. Avoid giant JSX ternaries and chained mutation handlers. Extract a cohesive dialog or section when it has its own inputs, lifecycle or workflow.
6. Keep remote effects in hooks and network mechanics in services. Retain transaction/permission decisions on the server; frontend visibility hints are not authorization.

The 400-line limit excludes blank lines and comments. It is a signal to split responsibilities, not an invitation to compress code. Existing feature pages are now expanded and divided into smaller modules; do not return to single-line components or multi-statement declarations.

## Types, effects and lists

Prefer an explicit interface for a new endpoint response and use `api<ResponseType>()` or `useData<ResponseType>()`. Use `unknown` at an unvalidated boundary. Existing configurable endpoints still use the shared `Row` type, with one documented `any` exception at that legacy boundary; do not create more scattered exceptions.

Effects declare their dependencies and clean up the exact listener/timer/subscription they created. Capture stable ref tokens inside cleanup setup. A dialog's event listener must call the latest close callback without reinitializing focus trapping every time its parent renders. Avoid suppressing dependency rules to hide these problems.

Use stable database IDs for list keys, or a stable value for a fixed enum. Never use a changing count, editable value, display label or render-time random value as an identity. Draft question/filter rows without server IDs may use positional keys only when their controls are controlled and they do not own independent item state; add persistent client identities before introducing per-row state or richer reordering. Native buttons always declare `type="button"` or `type="submit"`.

The shared form keeps select inputs uncontrolled and explicitly marks their initial option. A required picker starts on its disabled placeholder; it must not silently pick the first real recipient. This behavior has a regression test.

## Linting and formatting

From `apps/web` on a machine with Node **22.13+**:

```sh
npm ci
npm run lint
npm run format:check
npm run typecheck
npm test
npm run build
```

`npm run check` combines lint, formatting, typecheck and tests. `npm run lint:fix` fixes safe lint issues such as import organization; `npm run format` applies Prettier. Run both after larger edits, then inspect the diff. Formatting is separate from correctness linting so neither tool fights the other.

The ESLint flat configuration uses TypeScript recommended rules, explicit type imports, unused-import removal, predictable import/export ordering, hook rules/dependencies, list keys, explicit button types, unsafe-link/HTML checks and the module-size guard. It uses JSX rules compatible with Preact's DOM conventions, without requiring React-style `className` or the React compiler. Warnings fail the command. These checks complement, rather than replace, accessibility and browser testing.

Prettier defines two-space indentation, semicolons, single quotes in JavaScript/TypeScript, trailing commas, LF endings and a 100-column target. Original branding files are excluded so their source bytes stay intact. `.editorconfig` supplies matching editor defaults without requiring an editor extension or global configuration.

The frontend lockfile is checked in. A `minimatch` 11.0.1 override addresses a transitive lint-tool advisory until upstream ranges catch up; review it on future tooling upgrades. The current installed dependency graph reports no npm audit vulnerabilities. Vitest was updated to 5.1 to remove the older test pool's critical advisory. Do not use `npm audit fix --force` to bypass peer compatibility or roll tools backward.

## Project-contained tooling on the coding machine

The user authorized frontend tools only if their files can be removed with the project. Portable Node lives under `.tools/`, dependencies under `apps/web/node_modules/`, and all npm configuration, caches, logs, and temporary files under `.tools/`. No global package, user npm configuration, or permanent PATH change is needed.

Use a disposable PowerShell process from the repository root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/web.ps1 run lint:fix
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/web.ps1 run format
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/web.ps1 run check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/web.ps1 run build
```

The wrapper does not download tools automatically and fails clearly if the portable runtime is absent. On another machine use Docker or its own explicitly provisioned Node installation. Do not commit `.tools/`, `node_modules/`, caches or generated `dist/`; commit source configuration and `package-lock.json`. Close running project processes before deleting the folder.

## Docker and CI

Both use `npm ci` and the lockfile. The release web image runs lint and formatting checks before building static assets. Test Compose and the web CI job run `npm run check` and the build. This keeps source formatting and hook/JSX rules enforceable after moving the repository to the Docker machine.

This pass executed frontend lint, formatting, TypeScript, 78 component/helper/service-worker tests and the Vite build locally. Backend compilation, Docker startup, live authentication, visual acceptance and phone delivery remain separate work.
