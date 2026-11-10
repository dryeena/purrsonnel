export interface InstallPrompt extends Event {
  prompt(): Promise<void>;
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>;
}
let installPrompt: InstallPrompt | null = null;
export function pendingInstall() {
  return installPrompt;
}
export function clearInstall() {
  installPrompt = null;
  window.dispatchEvent(new Event('purrsonnel:install'));
}
export function isStandalone() {
  return (
    window.matchMedia('(display-mode: standalone)').matches ||
    (navigator as Navigator & { standalone?: boolean }).standalone === true
  );
}
export function isAppleMobile() {
  return (
    /iPad|iPhone|iPod/.test(navigator.userAgent) ||
    (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1)
  );
}
window.addEventListener('beforeinstallprompt', (event) => {
  event.preventDefault();
  installPrompt = event as InstallPrompt;
  window.dispatchEvent(new Event('purrsonnel:install'));
});
window.addEventListener('appinstalled', clearInstall);

let registrationPromise: Promise<ServiceWorkerRegistration> | undefined;
export function registerWorker(): Promise<ServiceWorkerRegistration> {
  if (!window.isSecureContext || !('serviceWorker' in navigator))
    return Promise.reject(new Error('Use a secure connection to enable home-screen features.'));
  // Wait for activation, including first registration; never time out into a fake enabled state.
  registrationPromise ??= navigator.serviceWorker
    .register('/sw.js', { scope: '/', updateViaCache: 'none' })
    .then(async (registration) => {
      if (registration.active) return registration;
      await navigator.serviceWorker.ready;
      return registration;
    })
    .catch((error) => {
      registrationPromise = undefined;
      throw error;
    });
  return registrationPromise;
}
export function vapidBytes(encoded: string): Uint8Array<ArrayBuffer> {
  const base64 = encoded.replaceAll('-', '+').replaceAll('_', '/');
  const binary = atob(base64 + '='.repeat((4 - (base64.length % 4)) % 4));
  const result = new Uint8Array(new ArrayBuffer(binary.length));
  for (let index = 0; index < binary.length; index++) result[index] = binary.charCodeAt(index);
  return result;
}
