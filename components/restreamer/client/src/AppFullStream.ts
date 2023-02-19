import AppFullStream from './components/AppFullStream.svelte';

const app = new AppFullStream({ target: document.body });

(window as any).app = app;
export default app;
