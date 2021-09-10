import AppRestreamer from './components/AppRestreamer.svelte';

const app = new AppRestreamer({ target: document.body });

(window as any).app = app;
export default app;
