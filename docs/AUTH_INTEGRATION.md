# Development authentication and future myFWA

Development requires **both** `APP_ENV=development` and `DEV_AUTH_ENABLED=true`. Startup rejects dev authentication in every other environment. Startup also rejects configurations with no implemented authentication provider, even if issuer/client keys are supplied. Production cannot accidentally start with an invented or unverified OIDC integration.

The browser sends `POST /api/v1/auth/dev/bootstrap` once, then `GET /api/v1/auth/me`. Bootstrap preserves valid existing sessions. A new session uses 256 random bits; only its hash is stored. Cookies are HttpOnly, SameSite=Lax, scoped to `/`, expire after 12 hours, and add Secure for an HTTPS origin. Identity switching deletes the previous session and returns a new CSRF token. There are compatibility aliases at `/api/auth/*`.

The API requires the configured loopback Host on non-health routes. All mutations require the configured Origin, reject cross-site fetch metadata, and (except bootstrap) verify a session-bound CSRF header. Bootstrap is Origin-protected. No wildcard CORS layer is enabled. Mutations are rate-limited per session; bootstrap is bounded process-wide. The current limiter is intentionally single-process and resets on restart. Do not make this development API public.

`AuthProvider` and `DevAuthProvider` establish the provider boundary. `MyFwaOidcProvider` is an intentionally unavailable configuration scaffold. No myFWA network calls, URLs, scopes, token formats, or claim names have been invented.

Environment keys reserved for the eventual adapter:

| Key | Purpose |
| --- | --- |
| `MYFWA_OIDC_ISSUER` | Official expected issuer/discovery identity |
| `MYFWA_OIDC_CLIENT_ID` | Registered client identifier |
| `MYFWA_OIDC_CLIENT_SECRET` | Server-only credential, if required by the official client type |
| `MYFWA_OIDC_REDIRECT_URI` | Exact registered callback |
| `MYFWA_OIDC_SCOPES` | Confirmed provider scopes |
| `MYFWA_OIDC_CLAIMS_MAPPING` | Confirmed stable identity and user-info claim mapping |

Before enabling the adapter: obtain provider documentation; implement state/nonce storage with single use and expiry, PKCE, exact issuer/audience checks, signature/JWKS and expiry validation, callback replay defense, secure session establishment, error paths, refresh/revocation behavior where documented, and verified ownership before linking any pending profile. A typed myFWA ID is never proof of identity. Keep stable local UUIDs separate from provider IDs. Test collision, merge, and account takeover cases before deployment.

Dev identities are obviously fake `DEV-FWA-*` values. The seed function exists in migrations but is called only after development configuration validation. There is no production seed call, login screen, password database, or default production administrator.
