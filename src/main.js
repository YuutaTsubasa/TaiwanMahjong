import { mount } from 'svelte';
import './app.css';
import './themes.css';
import './concept-ui.css';
import './ornaments.css';
import App from './App.svelte';

const app = mount(App, { target: document.getElementById('app') });

export default app;
