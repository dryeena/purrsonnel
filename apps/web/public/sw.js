/* Public fallback assets only. Never cache application/API responses or tokens. */
const PUBLIC_CACHE = 'purrsonnel-public-v1';
const PUBLIC_ASSETS = [
  '/offline.html',
  '/offline.css',
  '/branding/favicon.svg',
  '/branding/icon-192.png',
];
self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(PUBLIC_CACHE).then((cache) => cache.addAll(PUBLIC_ASSETS)));
});
self.addEventListener('activate', (event) => {
  event.waitUntil(
    Promise.all([
      caches
        .keys()
        .then((keys) =>
          Promise.all(
            keys
              .filter((key) => key.startsWith('purrsonnel-public-') && key !== PUBLIC_CACHE)
              .map((key) => caches.delete(key)),
          ),
        ),
      self.clients.claim(),
    ]),
  );
});
self.addEventListener('message', (event) => {
  if (event.data?.type === 'ACTIVATE_UPDATE') self.skipWaiting();
});
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (
    event.request.method !== 'GET' ||
    url.origin !== self.location.origin ||
    url.pathname.startsWith('/api') ||
    url.pathname.startsWith('/health')
  )
    return;
  if (event.request.mode === 'navigate') {
    event.respondWith(
      fetch(event.request).catch(
        async () => (await caches.match('/offline.html')) || Response.error(),
      ),
    );
  } else if (PUBLIC_ASSETS.includes(url.pathname) && !url.search) {
    event.respondWith(caches.match(url.pathname).then((cached) => cached || fetch(event.request)));
  }
});
function requestUrl(raw) {
  try {
    const url = new URL(raw, self.location.origin);
    const id = url.searchParams.get('id');
    if (
      url.origin === self.location.origin &&
      url.pathname === '/requests' &&
      /^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i.test(id || '')
    )
      return `/requests?id=${id}`;
  } catch {
    /* Fall through to the inbox. */
  }
  return '/requests';
}
self.addEventListener('push', (event) => {
  let payload = {};
  try {
    payload = event.data?.json() || {};
  } catch {
    /* Always show a generic visible notification. */
  }
  event.waitUntil(
    self.registration.showNotification('Purrsonnel request', {
      body:
        payload.kind === 'request_reminder'
          ? 'A request still needs attention in Purrsonnel.'
          : payload.kind === 'request_update'
            ? 'A request has an update in Purrsonnel.'
            : payload.kind === 'request_escalated'
              ? 'An unclaimed request needs your attention in Purrsonnel.'
              : 'A new request is waiting in your inbox.',
      icon: '/branding/icon-192.png',
      badge: '/branding/icon-192.png',
      tag: /^request-[0-9a-f-]{36}$/i.test(payload.tag || '') ? payload.tag : 'purrsonnel-request',
      renotify: false,
      silent: false,
      vibrate: [200, 100, 200],
      data: { url: requestUrl(payload.url) },
    }),
  );
});
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const destination = new URL(requestUrl(event.notification.data?.url), self.location.origin).href;
  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then(async (windows) => {
      for (const client of windows) {
        if (new URL(client.url).origin === self.location.origin) {
          // Let the application preserve an unsaved form before navigating.
          client.postMessage({ type: 'OPEN_REQUEST', url: destination });
          await client.focus();
          return;
        }
      }
      await self.clients.openWindow(destination);
    }),
  );
});
