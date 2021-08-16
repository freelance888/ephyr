import AppMix from './AppMix.svelte';

const app = new AppMix({ target: document.body });

(window as any).app = app;
export default app;
