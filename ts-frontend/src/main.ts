import { mount } from "svelte";
import "./style.css";
import "./pico.yellow.css";

import App from "./components/App.svelte";

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
