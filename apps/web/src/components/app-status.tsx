import { useEffect, useState } from 'preact/hooks';
import { route } from 'preact-router';

import { registerWorker } from '../lib/pwa';
import { confirmNavigation } from '../lib/unsaved';
export function AppStatus() {
  const [offline, setOffline] = useState(!navigator.onLine);
  const [waiting, setWaiting] = useState<ServiceWorker | null>(null);
  useEffect(() => {
    let alive = true;
    const online = () => setOffline(false);
    const off = () => setOffline(true);
    window.addEventListener('online', online);
    window.addEventListener('offline', off);
    let registration: ServiceWorkerRegistration | undefined;
    let installing: ServiceWorker | null = null;
    const state = () => {
      if (alive && registration?.waiting) setWaiting(registration.waiting);
    };
    const update = () => {
      installing = registration?.installing || null;
      installing?.addEventListener('statechange', state);
    };
    void registerWorker()
      .then((r) => {
        if (!alive) return;
        registration = r;
        state();
        r.addEventListener('updatefound', update);
      })
      .catch(() => {});
    const message = (event: MessageEvent) => {
      if (
        event.source !== navigator.serviceWorker.controller ||
        event.data?.type !== 'OPEN_REQUEST'
      )
        return;
      try {
        const url = new URL(event.data.url);
        if (url.origin === location.origin && url.pathname === '/requests' && confirmNavigation())
          route(url.pathname + url.search);
      } catch {
        // Ignore malformed notification URLs; navigation must stay on the current page.
      }
    };
    navigator.serviceWorker?.addEventListener('message', message);
    return () => {
      alive = false;
      window.removeEventListener('online', online);
      window.removeEventListener('offline', off);
      registration?.removeEventListener('updatefound', update);
      installing?.removeEventListener('statechange', state);
      navigator.serviceWorker?.removeEventListener('message', message);
    };
  }, []);
  return (
    <>
      {offline && (
        <div class="connection-strip" role="status">
          You’re offline. Reconnect before sending requests or saving changes. Anything already
          typed stays here while this page is open.
        </div>
      )}
      {waiting && (
        <div class="update-strip" role="status">
          <span>A new version of Purrsonnel is ready.</span>
          <button
            type="button"
            class="text-link"
            onClick={() => {
              if (!confirmNavigation()) return;
              navigator.serviceWorker.addEventListener(
                'controllerchange',
                () => location.reload(),
                { once: true },
              );
              waiting.postMessage({ type: 'ACTIVATE_UPDATE' });
            }}
          >
            Update now
          </button>
        </div>
      )}
    </>
  );
}
