const forms = new Set<symbol>();
export function trackUnsaved(key: symbol, dirty: boolean) {
  if (dirty) forms.add(key);
  else forms.delete(key);
}
export function hasUnsavedChanges() {
  return forms.size > 0;
}
export function confirmNavigation() {
  return (
    !hasUnsavedChanges() ||
    window.confirm('You have unsaved changes. Leave this page and discard them?')
  );
}
export function watchUnsavedChanges() {
  window.addEventListener('beforeunload', (event) => {
    if (hasUnsavedChanges()) {
      event.preventDefault();
      event.returnValue = '';
    }
  });
  document.addEventListener(
    'click',
    (event) => {
      if (
        event.defaultPrevented ||
        event.button !== 0 ||
        event.ctrlKey ||
        event.metaKey ||
        event.shiftKey ||
        event.altKey
      )
        return;
      const link = event.target instanceof Element ? event.target.closest('a') : null;
      if (
        !link ||
        link.target === '_blank' ||
        link.hasAttribute('download') ||
        !link.getAttribute('href') ||
        link.getAttribute('href')?.startsWith('#')
      )
        return;
      if (new URL(link.href, location.href).href === location.href) return;
      if (!confirmNavigation()) {
        event.preventDefault();
        event.stopPropagation();
      }
    },
    true,
  );
}
