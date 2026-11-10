import type { Row } from '../types/api';

export function csvCell(value: unknown): string {
  let text = String(value ?? '');
  if (/^[\s]*[=+@\-\t\r]/.test(text)) text = `'${text}`;
  return `"${text.replaceAll('"', '""')}"`;
}

export function downloadJson(name: string, value: unknown) {
  const url = URL.createObjectURL(
    new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' }),
  );
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function downloadCsv(name: string, rows: Row[], fields: string[]) {
  const text = [
    fields.map(csvCell).join(','),
    ...rows.map((r) =>
      fields.map((f) => csvCell(Array.isArray(r[f]) ? r[f].join(' · ') : r[f])).join(','),
    ),
  ].join('\r\n');
  const url = URL.createObjectURL(new Blob(['\ufeff', text], { type: 'text/csv;charset=utf-8' }));
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
