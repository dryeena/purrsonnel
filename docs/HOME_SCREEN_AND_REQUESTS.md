# Home screen, requests, and phone alerts

This pass adds code for an installable web app, a scoped request inbox, and real Web Push delivery. Nothing was installed, no push keys were generated, no subscription was created, and no phone alert was sent on the coding machine. Runtime and physical-device verification remain required.

## For people using Purrsonnel

- Open **Requests** from navigation or the mobile bottom bar. Choose **Send a request**, then a team or person, a specific title, details, and optional needed-by time. Teams show the actual recipient count, excluding the sender.
- A team request reaches a snapshot of its active members. One recipient chooses **I’m on it**. Others then see the assignee. The assignee or sender can mark it complete; only the sender can cancel. Two people cannot claim it concurrently.
- The inbox has separate **For me & my teams** and **Sent by me** views with server-side pagination. Opening a notification takes you to the request. Request contents remain inside the authenticated workspace.
- **Add to home screen** is available in the sidebar and relevant phone-alert help. Where supported, the button opens the native install dialog; otherwise it provides browser-specific instructions. An installed app hides the install action.
- On iPhone/iPad, add the site to the Home Screen, open that app, then explicitly tap **Enable phone alerts**. Permission must follow a user interaction. See [WebKit’s platform requirements](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/) and [Apple’s Home Screen instructions](https://support.apple.com/guide/iphone/bookmark-a-website-iph42ab2f3a7/ios).
- On supported Android/desktop browsers, open Requests and choose **Enable phone alerts**. The browser controls whether it prompts. Notifications can be disabled on the same device from the same screen.
- An alert asks for a short vibration and uses the system notification presentation. The device/browser controls vibration support, sound, silent mode, Focus, and permission. A web app cannot force a buzz. See [notification options](https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerRegistration/showNotification).

## Configure on the real machine

1. Run the existing Compose and test commands. The new Rust push-encryption dependency requires OpenSSL development libraries for native builds; the API Dockerfile includes build dependencies and the runtime library. No additional queue service is needed.
2. With Node already available there, run `node scripts/generate-vapid.mjs`. The command prints a new P-256 VAPID key pair for the operator. Keep the private key in `.env` or your secret environment, never in source or logs. This command was not run here.
3. Set `WEB_PUSH_ENABLED=true`, `VAPID_PUBLIC_KEY`, `VAPID_PRIVATE_KEY`, and `VAPID_SUBJECT` (a real `mailto:` operator contact or HTTPS contact URL). Compose passes these to the API. Startup verifies that the keys match and fails clearly if enabled configuration is invalid.
4. Serve the app on a secure origin. Localhost is suitable for desktop development. A real phone cannot reach the development computer by using its own `localhost`; remote phone testing needs a suitable secure host and authentication. The existing development stub remains loopback-only and production startup still fails closed until real authentication is implemented. Home-screen support does not weaken those guards.
5. Install the app, enable notifications deliberately, then send a request from a different permitted identity/device. Check foreground and background delivery, locked-screen behavior, disabled permission, Focus/silent mode, request deep links, and device opt-out.

Push configuration is optional: an unconfigured server still saves requests and in-app notifications. It clearly reports that phone alerts are unavailable. The send confirmation reports how many recipients received an inbox entry and how many device deliveries were queued; it never claims that a phone actually buzzed.

## Implementation and privacy

`manifest.webmanifest` provides stable app identity, standalone display, 192/512 PNG icons, a maskable icon, and Requests/Volunteer shortcuts. Apple receives a 180px touch icon and appropriate metadata. `scripts/build-icons.ps1` rasterizes the provided polygon SVG with Windows-native drawing APIs; it does not install anything or modify the originals. Installability and browser support are described in [MDN’s PWA guide](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable).

The service worker stores only an offline-help document, its CSS, and public branding. It never caches HTML application pages, API responses, signatures, QR tokens, people, application answers, or notes. It does not queue mutations offline. An open page retains typed text in memory while disconnected and disables saving; closing/reloading cannot preserve that text. Explicit save confirmation and unsaved-form warnings reduce accidental loss. Background polling does not replace a loaded screen with a loading spinner.

Web Push uses VAPID and AES128GCM payload encryption through the [web-push library](https://docs.rs/web-push/0.11.0/web_push/), with a bounded-timeout HTTP client and redirects disabled. Subscription endpoints are restricted to known browser push-service hosts; keys/endpoints are never returned in normal lists or audit data. Browser subscriptions belong to authenticated users and cannot silently replace another active owner. Development identity switching disables the previous device subscription first.

Requests, recipient snapshots, inbox notifications, and push outbox records commit in one database transaction. A single-process PostgreSQL outbox worker leases jobs with `FOR UPDATE SKIP LOCKED`, sends at most four concurrently, retries failures with bounded backoff, abandons expired jobs after 24 hours, and disables subscriptions that return 404/410. There is no Redis or external broker. Retries are at least once; deterministic notification tags reduce duplicate presentation. A push-service acceptance is not a device-read receipt.

Lock-screen content is deliberately generic: **“A new request is waiting in your inbox.”** Details require an authenticated API read. Click destinations are restricted to same-origin request links. Existing windows receive a navigation message so an unsaved form can be kept. No action is accepted or completed directly by a notification click.

Permission `requests.send` is explicitly scoped to departments/divisions or global. Receiving a request does not require send authority. Personnel rank grants no request authority. Fictional HR receives a global grant, and fictional active department members receive department grants; ordinary new real users do not acquire that authority automatically. The sender and snapshotted recipients can read a request. An explicit, audited escalation may add its chosen person to that request only. Final state transitions are locked and audited. Idempotency binds a send key to its sender and exact payload, preventing duplicate inbox entries and push jobs.

## Device acceptance checklist

- Android Chrome: install, reopen standalone, send to a person and a team, enable/disable push, receive while backgrounded and locked.
- iPhone Safari: Home Screen install, launch installed app, gesture-based permission, receive while closed, tap back to the correct request.
- Desktop Chrome/Edge: native install where offered; manual instructions otherwise; correct icons and app title.
- No browser prompt until the user asks to install/enable alerts; denied permission produces useful instructions.
- Offline cold launch shows the public fallback; private pages/API calls are never cached or replayed.
- Unsaved edits survive background polling; navigation/dialog dismissal asks before discard; offline form input remains editable.
- Send retries create one request; simultaneous team claims produce one owner; another department cannot send or read out-of-scope requests.
- Confirm not configured, no subscribed recipients, queued, failed/expired delivery states, and iOS Focus/vibration limitations without implying guaranteed buzzing.

See [request follow-through and phone preferences](REQUEST_FOLLOW_THROUGH.md) for reminders, escalation, quiet hours, and delivery history.
