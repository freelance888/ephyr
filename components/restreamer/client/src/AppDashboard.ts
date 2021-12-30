import AppDashboard from './components/AppDashboard.svelte';

const app = new AppDashboard({ target: document.body });

(window as any).app = app;
export default app;
