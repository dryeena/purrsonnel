export function date(
  value: string | null | undefined,
  time = false,
  timezone = 'America/New_York',
) {
  if (!value) return '—';
  const parsed = new Date(value.length === 10 ? `${value}T12:00:00` : value);
  return new Intl.DateTimeFormat(
    'en-US',
    time
      ? {
          month: 'short',
          day: 'numeric',
          hour: 'numeric',
          minute: '2-digit',
          timeZone: timezone,
          timeZoneName: 'short',
        }
      : { month: 'short', day: 'numeric', year: 'numeric' },
  ).format(parsed);
}
