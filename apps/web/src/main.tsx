import './styles/app.css';
import './styles/polish.css';

import { render } from 'preact';

import { App } from './app';
import { registerWorker } from './lib/pwa';
import { watchUnsavedChanges } from './lib/unsaved';
watchUnsavedChanges();
void registerWorker().catch(() => {
  /* Install help and alert settings explain browser limitations. */
});
render(<App />, document.getElementById('app')!);
