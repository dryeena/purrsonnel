import { api } from '../services/api-client';

import { isAppleMobile, isStandalone, registerWorker, vapidBytes } from './pwa';
export function pushSupportMessage(): string | null {
  if (!window.isSecureContext) return 'Phone alerts need a secure connection.';
  if (isAppleMobile() && !isStandalone())
    return 'Add Purrsonnel to your Home Screen, open it from its icon, then enable phone alerts here.';
  if (!('Notification' in window) || !('PushManager' in window) || !('serviceWorker' in navigator))
    return 'This browser does not support phone alerts. Your requests are still available in the inbox.';
  if (Notification.permission === 'denied')
    return 'Notifications are blocked. Allow them in your browser or phone notification settings, then return here.';
  return null;
}
export async function enablePush(publicKey: string) {
  const limitation = pushSupportMessage();
  if (limitation) throw new Error(limitation);
  // Permission is requested directly from the click handler, before awaiting registration.
  const permission = await Notification.requestPermission();
  if (permission !== 'granted')
    throw new Error('Phone alerts were not enabled. You can continue using your inbox.');
  const registration = await registerWorker();
  const subscription =
    (await registration.pushManager.getSubscription()) ||
    (await registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: vapidBytes(publicKey),
    }));
  const json = subscription.toJSON();
  try {
    await api('/push/subscriptions', 'POST', { endpoint: json.endpoint, keys: json.keys });
  } catch (error) {
    await subscription.unsubscribe();
    throw error;
  }
  return subscription;
}
export async function disablePush() {
  if (!('serviceWorker' in navigator)) return;
  const registration = await navigator.serviceWorker.getRegistration('/');
  const subscription = await registration?.pushManager?.getSubscription();
  if (!subscription) return;
  // Revoke server delivery before detaching the browser subscription.
  await api('/push/subscriptions', 'DELETE', { endpoint: subscription.endpoint });
  await subscription.unsubscribe();
}
