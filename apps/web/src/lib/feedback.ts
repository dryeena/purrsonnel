export type NoticeTone = 'success' | 'info';
export interface Notice {
  message: string;
  tone: NoticeTone;
}
export function notify(message: string, tone: NoticeTone = 'success') {
  window.dispatchEvent(new CustomEvent<Notice>('purrsonnel:notice', { detail: { message, tone } }));
}
