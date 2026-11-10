import type { Row } from '../types/api';
export function minuteTime(minute: number): string {
  return `${Math.floor(minute / 60)
    .toString()
    .padStart(2, '0')}:${(minute % 60).toString().padStart(2, '0')}`;
}
export function timeMinute(value: string): number {
  if (!/^([01]\d|2[0-3]):[0-5]\d$/.test(value))
    throw new Error('Choose a valid time for quiet hours.');
  const [hour, minute] = value.split(':').map(Number);
  return hour * 60 + minute;
}
export function followUpPayload(values: Row) {
  const delay = (value: string) => {
    if (!value) return null;
    const minutes = Number(value);
    if (!Number.isInteger(minutes) || minutes < 15 || minutes > 43200)
      throw new Error('Choose a follow-up between 15 minutes and 30 days.');
    return minutes;
  };
  const reminder_after_minutes = delay(values.reminder_after),
    escalation_after_minutes = delay(values.escalation_after);
  if (escalation_after_minutes && !values.escalation_user_id)
    throw new Error('Choose a person to receive the escalation.');
  if (
    reminder_after_minutes &&
    escalation_after_minutes &&
    escalation_after_minutes <= reminder_after_minutes
  )
    throw new Error('Choose an escalation time after the reminder.');
  return {
    reminder_after_minutes,
    escalation_after_minutes,
    escalation_user_id: escalation_after_minutes ? values.escalation_user_id : null,
  };
}
export function reminderReady(request: Row, now = Date.now()): boolean {
  return (
    ['open', 'accepted'].includes(request.status) &&
    now - new Date(request.last_reminded_at || request.created_at).getTime() >= 15 * 60000
  );
}
export function requestDate(value: string | null | undefined): string {
  if (!value) return '—';
  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    timeZoneName: 'short',
  }).format(new Date(value));
}
