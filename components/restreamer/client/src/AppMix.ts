import AppMix from './components/AppMix.svelte';

const app = new AppMix({ target: document.body });

(window as any).app = app;
export default app;
