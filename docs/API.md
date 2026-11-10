# API contract

All feature routes are under `/api/v1`. Browser requests use same-origin cookie authentication; mutations require configured `Origin` and `X-CSRF-Token`. Use `Content-Type: application/json`. No request may provide an actor ID to impersonate the current principal, other than the explicit development-only switch endpoint. Errors use `{ "type":"about:blank", "title":"Conflict", "status":409, "detail":"…" }`.

`GET /health/live` checks process liveness; `GET /health/ready` checks PostgreSQL. Health responses contain no credentials. `/api-docs/openapi.json` generates the health schema through Utoipa; this document is the full authenticated feature reference. No endpoint in this reference implies runtime verification has passed.

List routes accept `page` (1-based) and `page_size` (1–100) where indicated. Users return `{items,total,page,page_size}`. Other feature lists return arrays; request the next page until fewer than `page_size` records are returned. Organization/configuration catalogs are bounded by their natural organization scope. Audit/bulk histories have explicit page/record limits. An API call that is not listed here is not implemented.

## Identity

| Method/path | Request | Response and checks |
| --- | --- | --- |
| `POST /auth/dev/bootstrap` | `{}` | `{csrf_token}`; dev only; Origin/Host validated; preserves existing session; creates HttpOnly cookie otherwise |
| `POST /auth/dev/switch` | `{user_id:"uuid"}` | Rotated cookie + CSRF; existing session/CSRF; seeded active identity only |
| `GET /auth/me` | — | `{user,grants,dev_mode,identities,csrf_token}`; active session required |
| `POST /auth/logout` | `{}` | Deletes session and expires cookie |

Compatibility aliases exist at `/api/auth/me`, `/api/auth/dev/bootstrap`, `/api/auth/dev/switch`. myFWA start/callback routes are deliberately absent until implemented.

## People and organization

| Method/path | Request or query | Checks / response |
| --- | --- | --- |
| `GET /users` | `q,department,division,level,status,volunteer,year,skill,manager,pending_terms,page,page_size,sort,order` | Own profile plus `people.read` scoped records; sort whitelist `display_name,department,status,verified_years`; order `asc/desc` |
| `GET /users/{id}` | — | Self or `people.read` scope; legal name/email only with `people.read_sensitive` |
| `PATCH /users/{id}` | `{display_name?,status?,volunteer_eligible?,personnel_level_id?,department_ids?,primary_department_id?,manager_id?,clear_manager?}` | `people.edit` scope; memberships/rank/manager require global HR; rejects unknown fields; graph cycle check, audit |
| `GET /users/{id}/directs` | — | Own directs or scoped person read |
| `GET /users/{id}/service-years` | — | Self or scoped person read; explicit records |
| `PUT /users/{id}/service-years/{year}` | `{verification_status:"verified"}` | Scoped `people.edit`; upsert; audit; also accepts `self_reported` |
| `DELETE /users/{id}/service-years/{year}` | — | Scoped `people.edit`; audit |
| `GET /skills` | — | Authenticated catalog |
| `POST /skills` | `{name:"Guest support"}` | Global `people.edit`; audit |
| `PUT /users/{id}/skills/{skill}` | `{}` | Scoped `people.edit`; idempotent assignment/audit |
| `DELETE /users/{id}/skills/{skill}` | — | Scoped `people.edit`; audit |
| `GET /organization/tree` | — | Authenticated department/division/level/position-level/event catalogs; scoped membership counts |
| `POST /departments` | `{name,slug,division_id?,description?,lead_user_id?}` | Global `organization.manage`; audit |
| `POST /divisions` | `{name,description?}` | Global `organization.manage`; audit |
| `GET /profile-fields` | — | Global `settings.manage`; field definitions |
| `POST /profile-fields` | `{key,label,type,choices?,visibility?,editable_by?,max_length?}` | Global `settings.manage`; controlled type/policies; defaults self visibility/HR editing; visibility `self/personnel/sensitive`, editor `self/hr` |
| `GET /users/{id}/profile-fields` | — | Self/scoped person read plus visibility restrictions |
| `PATCH /users/{id}/profile-fields` | `{values:{"definition-uuid":"value"}}` | Self for self-editable fields or scoped `people.edit`; server type/length validation; sensitive grant for other people’s sensitive fields; audit |
| `POST /levels` | `{label,slug,rank}` | Global `settings.manage`; unique rank/slug; audit |
| `PATCH /position-levels/{id}` | `{personnel_level_id:"uuid"}` | Global `settings.manage`; explicit mapping/audit |
| `GET /permission-grants` | — | Global `settings.manage`; catalog and assignments |
| `POST /permission-grants` | `{user_id,permission,scope_type,scope_id}` | Global `settings.manage`; whitelist and real scope validation; audit |
| `DELETE /permission-grants/{id}` | — | Global `settings.manage`; last active system admin protected; audit |

Department-filter membership is independent of primary membership. `verified_years` counts verified distinct rows. No years are inferred from elapsed time. Rank changes do not grant authority. CSV generation uses only already-authorized directory responses and escapes spreadsheet formula prefixes; page exports are labeled as page exports.

## Terms

| Method/path | Request | Checks / response |
| --- | --- | --- |
| `GET /terms/applicable` | — | Current applicable published versions, text/hash and own signature state |
| `GET /terms` | — | Global `terms.manage`; drafts and all versions |
| `POST /terms` | `{title,audience,body_markdown,minimum_personnel_rank?,requires_volunteer?}` | Global `terms.manage`; draft v1; audience `all/volunteers/personnel` |
| `POST /terms/{id}/versions` | `{body_markdown}` | Global `terms.manage`; new draft; no editing published evidence |
| `POST /terms/{id}/publish` | `{version_id}` | Global `terms.manage`; locks term; supersedes old version atomically |
| `POST /term-versions/{id}/sign` | `{typed_name,consent:true}` | Own authenticated user; current/applicable version only; immutable evidence; idempotent receipt |
| `GET /terms/compliance` | — | Rows filtered by `terms.read_compliance` scope, up to 1,000; never returns another person’s signature text |

## Volunteer time

| Method/path | Request | Checks / response |
| --- | --- | --- |
| `GET /volunteer/entries` | `page,page_size` | Own entries or staff scope; department/approver display names |
| `POST /volunteer/entries` | `{department_id,event_edition_id?,work_date,minutes_worked?,started_at?,ended_at?,work_description}` | `volunteer.time.submit`, linked active eligible person; exactly one duration mode; 1–960 minutes; overlap guard; draft |
| `POST /volunteer/entries/{id}/submit` | `{}` | Own draft; submit permission; audited transition |
| `POST /volunteer/entries/{id}/withdraw` | `{}` | Own draft/submitted entry; revokes codes |
| `POST /volunteer/entries/{id}/approval-qr` | `{}` | Own submitted entry; rotates opaque token; `{url,expires_at}` |
| `GET /volunteer/approval-preview` | `token=opaque-reference` | `volunteer.time.approve` in actual work department; no self approval; no mutation |
| `POST /volunteer/approval-decisions` | `{token,decision:"approve"|"deny",note,idempotency_key:"uuid"}` | Authenticated scoped staff; explicit request; expiry/status/self-rule rechecked under lock; deny requires reason; immutable event |
| `GET /volunteer/reports/hours` | — | Own totals plus permitted staff scope; event/person/department group |

Navigation-only staff QR uses `${window.location.origin}/volunteer` with no token. Approval URL uses `/volunteer/approve?t=…`; it conveys no permission. QR generation is local; no QR service receives tokens. Identical idempotency retries return success; reuse for a different payload conflicts.

## Positions, forms, applications, interviews

| Method/path | Request | Checks / response |
| --- | --- | --- |
| `GET /positions` | `page,page_size` | Open postings or `positions.manage` scope; question snapshot, redacted application count |
| `POST /positions` | `{title,department_id,position_level_id,is_leadership,description_markdown,hiring_owner_user_id,status,questions?}` | `positions.manage` scope; defaults merged into immutable form version; initial `draft/open`; owner needs finalization permission |
| `PATCH /positions/{id}` | `{status}` | `positions.manage` scope; controlled lifecycle status |
| `GET /default-questions` | — | Global `positions.manage`; default-question catalog |
| `POST /default-questions` | `{key,label,type,required,sensitive}` | Global `settings.manage`; controlled schema; future postings only |
| `GET /form-templates` | — | `positions.manage` scope; template/version library |
| `POST /form-templates` | `{name,department_id?,schema:[question,…]}` | `positions.manage` scope; immutable v1 |
| `POST /form-templates/{id}/versions` | `{schema:[question,…]}` | Scope check; new immutable version |
| `GET /applications` | `page,page_size` | Own or scoped `applications.review`; no answer bodies in list |
| `POST /applications` | `{position_posting_id,answers_json}` | `applications.submit`; open/nonexpired posting; own draft; duplicate active submission prevented |
| `GET /applications/{id}` | — | Own or scoped review; sensitive answers stripped without sensitive access |
| `PATCH /applications/{id}` | `{answers_json}` | Own draft only; bounded data |
| `POST /applications/{id}/submit` | `{}` | Own draft; rechecks posting and full schema/required answers; takes myFWA ID from current user |
| `GET /interviews` | `page,page_size` | Applicant, panel member, or scoped manager; panel response labels, no note bodies |
| `POST /interviews` | `{application_id,scheduled_start_at,scheduled_end_at,timezone,location_label,location_url?,panel:[user_id,…]}` | Scoped `interviews.manage`; timezone-aware times, real IANA timezone, HTTPS URL; panel requires explicit `interviews.decide` |
| `PUT /interviews/{id}/my-decision` | `{decision:"approve"|"deny",note}` | Assigned panel member + scoped `interviews.decide`; deny reason; never auto-finalizes |
| `GET /interviews/{id}/notes` | — | Author-only/panel/HR-sensitive visibility evaluated per note |
| `POST /interviews/{id}/notes` | `{body,visibility:"author_only"|"interview_panel"|"hr"}` | Assigned panel or scoped manager; bounded text; audit excludes body |
| `POST /applications/{id}/finalize` | `{decision:"approve"|"deny",reason,idempotency_key:"uuid"}` | Owner + scoped `interviews.finalize`, or global HR override; nonempty reason; transactional decision/membership/level/audit/notification |

Controlled questions use `{key,label,type,required,options?,max_length?,sensitive?}`. Supported types: text, textarea, number, date, boolean, single_select, multi_select, url. Keys are unique identifiers. No arbitrary HTML, SQL, script, or dynamic expression is executed. `Other` requires a configured position-level mapping before approval. Existing pending identities remain pending; finalization never fabricates a login.

## Lists, bulk changes, dashboard, audit

| Method/path | Request | Checks / response |
| --- | --- | --- |
| `GET /saved-lists` | — | Global `people.read`; current owner’s private lists |
| `POST /saved-lists` | `{name,filter_ast_json}` | Global `people.read`; whitelisted AST; private owner |
| `POST /saved-lists/{id}/run` | `{}` | Owner; current scope applied to each person; `{items,total}` |
| `DELETE /saved-lists/{id}` | — | Owner only |
| `POST /bulk-changes/preview` | `{filter,operation:{field:"status"|"volunteer_eligible",value}}` | Global `people.bulk_update`; scoped `people.edit` targets; <=250; stores exact snapshot/version; 30-minute expiry |
| `POST /bulk-changes/execute` | `{preview_id,confirmed:true}` | Same creator; one execution; rechecks edit access/version per person; conflicts reported; per-person audit |
| `GET /bulk-changes` | — | Global bulk permission; own 50 most recent jobs, outcomes |
| `GET /dashboard` | — | Principal-tailored counts; visible departments; own notifications |
| `GET /audit-events` | `page,page_size` | Global `audit.read`; append-only events; no private note/answer bodies |

Filter examples:

```json
{"op":"and","args":[{"op":"eq","field":"status","value":"active"},{"op":"or","args":[{"op":"contains","field":"department","value":"Registration"},{"op":"range","field":"verified_years","min":2,"max":5}]}]}
```

Allowed operations: `and/or` with `args`, `not` with `arg`, `eq/in/contains/range/is_null` with an approved `field`. Allowed fields: display_name, status, volunteer_eligible, level, department (primary department label), verified_years, myfwa_id. Lists are evaluated only on scoped directory records; nested depth <=8 and group size <=20. The current implementation rejects scopes over 1,000 people rather than silently truncating. Bulk updates do not accept personnel roles, permission grants, or user-supplied SQL.

## Lifecycle, detail, and notification routes

| Method/path | Request | Checks / response |
| --- | --- | --- |
| `POST /users` | `{display_name,email?,position_posting_id?,unverified_myfwa_id?,answers_json?}` | Global `people.edit`; optional scoped application review; validates required form answers for intake; creates only pending identity. Supplied myFWA ID stays unverified application evidence, never a linked identity |
| `GET /departments`, `GET /divisions`, `GET /levels` | — | Authenticated catalogs; scoped department counts |
| `GET /departments/{id}` | — | Authenticated department metadata; scope-redacted count |
| `PATCH /departments/{id}` | `{name?,description?,lead_user_id?,clear_lead?,active?}` | `organization.manage` scope; audited; lead does not create manager relationships |
| `PATCH /divisions/{id}` | Same as department | Global organization management; audited |
| `GET /users/{id}/export` | — | Global sensitive access + scoped person read; audited administrator-assisted JSON record export |
| `GET /positions/{id}` | — | Open or scoped position management; form snapshot |
| `POST /positions/{id}/owner` | `{hiring_owner_user_id,reason}` | Scoped position management; new owner needs finalization capability; audit includes before/after authority |
| `GET /volunteer/entries/{id}` | — | Own entry or scoped approver/admin |
| `PATCH /volunteer/entries/{id}` | Same payload as creation | Own draft only; validates duration/overlap; audited |
| `POST /volunteer/entries/{id}/reopen` | `{reason}` | Scoped time admin; approved/rejected only; saves immutable full snapshot, preserves decisions, revokes unused references, returns to draft |
| `GET /interviews/{id}` | — | Applicant, assigned panel, or scoped manager; no note bodies |
| `PATCH /interviews/{id}` | `{scheduled_start_at,scheduled_end_at,timezone,location_label,location_url?,status}` | Scoped interview management; schedule validation; `scheduled/completed/cancelled`; audit |
| `POST /interviews/{id}/interviewers` | `{interviewer_user_id}` | Scoped manager; explicit panel-decision permission required; audited/idempotent |
| `GET /saved-lists/{id}` | — | Owner or HR-shared/global people read |
| `PATCH /saved-lists/{id}` | `{name,filter_ast_json,shared_scope:"private"|"hr"}` | Owner; validated AST; HR sharing requires global read; audit |
| `GET /bulk-changes/{id}` | — | Creator + global bulk permission; stored outcomes |
| `GET /notifications` | `page,page_size` | Own notifications only |
| `POST /notifications/{id}/read` | `{}` | Recipient only; idempotent read acknowledgment |

Position creation also accepts optional `form_template_version_id` from an organization-wide or same-department template. New templates automatically snapshot organization defaults. GET saved lists includes HR-shared lists for global readers; executing any list still applies current record scope. The notification bell reads the actual notification store. Volunteer entry lists refresh every 15 seconds while visible, in addition to immediate refresh after local decisions.

## Requests and Web Push

| Method/path | Request | Checks / response |
| --- | --- | --- |
| `GET /requests/recipients` | — | Authenticated; department/person catalog filtered by explicit `requests.send` scope; counts exclude sender |
| `GET /requests` | `view=all/inbox/sent,status=all/active/open/accepted/completed/cancelled,q,page,page_size` | Sender or snapshotted recipient only; all filters before pagination; `active` means open/accepted; search title/body, max 160 characters, literal wildcard escaping; redacted push-delivery counts for sender |
| `POST /requests` | `{recipient_user_id?,recipient_department_id?,title,body,urgency:"normal"|"urgent",due_at?,idempotency_key}` | Exactly one audience; scoped send capability; active recipients; 1–250 recipient snapshot; transactional request/inbox/outbox/audit; idempotent exact-payload retry |
| `GET /requests/{id}` | — | Sender or snapshot recipient only; no automatic mutation on read |
| `POST /requests/{id}/respond` | `{action:"accept"|"complete"|"cancel"}` | Recipient claims open request; assignee/sender completes accepted request; sender cancels open/accepted request; locked transition and audit |
| `GET /push/config` | — | Authenticated; `{enabled,public_key}` only; never private key |
| `POST /push/subscriptions` | `{endpoint,keys:{p256dh,auth}}` | Current principal; configured server; HTTPS push-host allowlist; key validation; no silent active-account reassignment |
| `DELETE /push/subscriptions` | `{endpoint}` | Current owner only; disables device and expires pending jobs |
| `POST /push/subscriptions/status` | `{endpoint}` | Read-only authenticated/CSRF-protected device status; active only for the current owner on a configured server; no keys/other owner disclosed |

Notifications use types `service_request`, `request_update`, `request_reminder`, and `request_escalated`, with a request UUID and same-origin URL. Request text is excluded from audit and phone push payloads. Create returns `{id,recipient_count,push_queued,push_suppressed,push_enabled}`; replay returns `{id,replayed:true}`. Queued delivery may wait for quiet hours and does not guarantee device sound, vibration, or a read receipt. No request action is performed by a GET or a notification click.

## Follow-through and account phone preferences

| Method/path | Request | Checks / response |
| --- | --- | --- |
| `GET /notification-preferences` | — | Own preferences only; defaults `request_alerts=all,quiet_enabled=false,timezone=UTC,quiet_start=1320,quiet_end=480` |
| `PUT /notification-preferences` | `{request_alerts:"all"|"urgent"|"off",quiet_enabled,timezone,quiet_start,quiet_end}` | Own account; IANA timezone validated against PostgreSQL, minutes 0–1439 and start/end different; audited; wakes pending jobs for policy reevaluation |
| `GET /requests/{id}/follow-up` | — | Sender/recipient; `{follow_up,activity,delivery}` with latest 100 immutable events, delivery aggregate only for sender; no subscription details |
| `PUT /requests/{id}/follow-up` | `{reminder_at?,escalation_at?,reminder_after_minutes?,escalation_after_minutes?,escalation_user_id?}` | Active request's sender only; future 15 minutes–30 days; escalation time/person paired, escalation after reminder; explicit scoped active target; all-null clears; replaces pending schedule, retained history |
| `POST /requests/{id}/remind` | `{}` | Sender only, open/accepted; at least 15 minutes from creation/last reminder; open targets active audience, accepted targets assignee; transactional inbox/outbox/activity/audit |

Scheduled escalation is open-only, rechecks current sending scope and account statuses, and adds only the chosen person to that request's audience. The scheduler runs independently of phone configuration. Phone policy never suppresses inbox delivery. Alert jobs expire after 24 hours; quiet-hour waiting consumes no delivery attempts. See [follow-through design and behavior](REQUEST_FOLLOW_THROUGH.md).

Follow-up steps accept either an absolute UTC timestamp or relative integer minutes, never both for the same step. Relative delays use one server timestamp and avoid device-clock drift or network latency at the 15-minute minimum. The UI submits relative minutes.
