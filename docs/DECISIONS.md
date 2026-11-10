# Decisions

- Follow the requested Preact/TypeScript/Vite and Rust/Axum/SeaORM stack (SeaORM replaces the original direct SQLx layer at the user's request). Keep PostgreSQL in tests and development. No substitutes or extra infrastructure.
- Respect the coding-only environment: write source and future-machine commands, install nothing. Record unrun checks accurately.
- Preserve the provided brand assets byte for byte. Use a restrained plum/lavender palette, neutral surfaces, readable spacing, line icons, and a responsive left navigation. Keep member priorities separate from HR metrics. Use system typography, with no third-party font calls.
- Use ordinary Preact hooks and one fetch abstraction rather than adding a React compatibility layer solely for Query. This is allowed by the spec's minimal-state intent and keeps the extension boundary small.
- Use Tailwind's Vite integration and a local CSS design system for consistent reusable components. No large component framework.
- Store no bearer credential in browser persistence. Sessions and permissions remain server-side, with a session-bound CSRF token in browser memory.
- Do not infer authority from organizational rank, department head, QR possession, panel majority, or a typed provider ID.
- Render policy/description text safely as preformatted plain text, preserving Markdown source as evidence. Rich sanitized Markdown rendering is a future improvement, not an HTML injection surface.
- Published application form versions are always immutable, which is stricter and simpler than freezing only after first submission.
- Keep bulk operations intentionally narrow: reviewed status/eligibility changes, up to 250 records, with conflict reporting. More operations require their own tests. Lists currently reject scopes above 1,000 people until a parameterized SQL AST compiler is introduced.
- In-app notifications persist independently of any email provider. No SMTP dependency or pretend email delivery.
- No jurisdiction-specific legal assertions: demo terms are fictional and require review before operational use. No license selected.
