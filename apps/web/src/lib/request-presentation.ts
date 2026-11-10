import type { Row } from '../types/api';
export function isRequestOverdue(request: Row, now = Date.now()) {
  return (
    ['open', 'accepted'].includes(request.status) &&
    !!request.due_at &&
    new Date(request.due_at).getTime() < now
  );
}
export function relativeTime(value: string, now = Date.now()) {
  const seconds = (new Date(value).getTime() - now) / 1000;
  if (!Number.isFinite(seconds)) return 'Date unavailable';
  if (Math.abs(seconds) < 60) return seconds > 0 ? 'In a moment' : 'Just now';
  const [amount, unit]: [number, Intl.RelativeTimeFormatUnit] =
    Math.abs(seconds) < 3600
      ? [seconds / 60, 'minute']
      : Math.abs(seconds) < 86400
        ? [seconds / 3600, 'hour']
        : Math.abs(seconds) < 604800
          ? [seconds / 86400, 'day']
          : [seconds / 604800, 'week'];
  return new Intl.RelativeTimeFormat('en-US', { numeric: 'auto' }).format(Math.round(amount), unit);
}
