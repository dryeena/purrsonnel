# Request follow-through and phone preferences

This feature pass is local source work. Nothing was installed, launched, migrated, connected, or sent. Runtime and phone checks remain deferred.

## Sending and following up

Send a request to a person or a department, then open **View request & follow-up options**. The original audience is snapshotted. One recipient can accept; reminders then go only to the assignee. Completing or cancelling stops future follow-ups. Everyone on the request receives an inbox update after acceptance, completion, or cancellation, excluding the person who made that change.

The sender can send a manual reminder after 15 minutes from creation or the last reminder. The same cooldown applies to automatic reminders. Each reminder creates inbox evidence, even with phone delivery disabled. There is no promise of a buzz or a response.

**Schedule follow-up** provides one reminder and one optional escalation. Choose relative times from 15 minutes to a day in the UI; the API supports times up to 30 days away. Escalation must follow the reminder when both are scheduled. Relative delays are calculated with the server clock, so device-clock drift does not move the schedule. Saving replaces the current schedule; removing a schedule clears pending steps without deleting its history. Each scheduled step runs once.

Escalation is explicit: choose an active person within your sending scope. The UI explains that this will grant access to the request details. Only if the request is still open does the scheduler add that person to the audience and notify them. It rechecks the sender's access and both accounts' active status. Accepted or closed requests and unavailable/out-of-scope escalation targets produce a skipped event instead of an alert. There is no inferred manager, team-wide escalation, or automatic reassignment of accepted work.

The scheduler runs every 30 seconds within the API, even with Web Push disabled. It processes at most 50 requests per cycle. It locks requests with PostgreSQL `FOR UPDATE SKIP LOCKED`, matching interactive lock order; inbox notifications, phone jobs, processed markers, activity, and escalation recipient additions commit together. Missed due steps are processed after restart while still relevant. The sender is the audit authority for an explicitly configured escalation; activity records distinguish automatic actions with a null actor.

## Phone preferences

Open **Alert preferences & quiet hours** on Requests. These are account-wide settings for all subscribed devices:

- All request phone alerts, urgent requests and their updates only, or inbox only.
- Optional daily quiet hours, including periods crossing midnight.
- A fixed IANA timezone such as `America/Denver`, independent of later device travel. The browser timezone is offered for the initial unsaved preference form. PostgreSQL validates timezone names and resolves daylight saving transitions.

Urgent requests respect quiet hours. Inbox delivery is never held back. Phone jobs are evaluated at enqueue and again before a send/retry. Quiet-hour jobs wait without consuming transport attempts. Jobs expire after 24 hours; alerts for obsolete new/closed requests are discarded before transport. Opted-out jobs stay suppressed when alerts are re-enabled, avoiding a backlog flood. Changes wake pending jobs to reevaluate current preferences. A send already handed to the push service cannot be recalled; the phone OS controls actual presentation.

**Turn alerts off** detaches this device. **Inbox only** changes all devices' account policy. Neither removes inbox notifications or existing request records. Device subscriptions and account policy are separate, deliberate controls.

## Delivery and history

Senders see device-job totals: accepted by the push service, queued (including waiting/retrying), suppressed by preferences, and failed/expired. Totals include follow-up alerts, not unique people. They expose no endpoints, encryption keys, device ownership, or recipient preference records. Push-service acceptance is not delivery to the phone or proof of reading. There is no read-tracking promise.

Participants see the latest 100 immutable request activity events: sent, accepted, completed, cancelled, schedule changes, reminders, escalation, and skipped steps. Scheduled times are shown separately from actual event timestamps. Earlier evidence remains in the database. Requests created before this migration have no invented historical events.

`alert_preferences.rs`, `request_follow_up.rs`, and `push.rs` keep policy, scheduling, and transport separate. New channels should consume persisted notification records and apply an explicit channel policy; new reminder cadence or escalation tiers should extend the schedule model and audited access checks, rather than run timers in the browser.
