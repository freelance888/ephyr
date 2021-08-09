import App from './App.svelte';
import Toolbar from './Toolbar.svelte';
import PageAll from './pages/All.svelte';
import ClientGqlContext from './ClientGqlContext.svelte';

const app = new App({
  target: document.body,
  props: {
    mainComponent: PageAll,
    toolbarComponent: Toolbar,
    graphQlContext: ClientGqlContext
  },
});

(window as any).app = app;
export default app;
