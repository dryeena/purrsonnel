/** Converts stored identifiers into readable UI text. Keep machine keys out of the interface. */
export function humanize(value: string) {
  const words = value.replaceAll('_', ' ').replaceAll('.', ' ').trim();
  return words ? words[0].toUpperCase() + words.slice(1) : '';
}
