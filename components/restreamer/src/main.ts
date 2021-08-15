import AppClient from './AppClient.svelte';

const app = new AppClient({target: document.body});

(window as any).app = app;
export default app;
