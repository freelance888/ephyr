import App from './App.svelte';
import Toolbar from './Toolbar.svelte';
import PageAll from './pages/All.svelte';
import { createGraphQlClient } from './util';

let isOnline = true;
const gqlClient = createGraphQlClient('/api', () => isOnline = true, () => isOnline = false);

const app = new App({
  target: document.body,
  props: {
    mainComponent: PageAll,
    toolbarComponent: Toolbar,
    gqlClient,
    isOnline
  },
});

(window as any).app = app;
export default app;
