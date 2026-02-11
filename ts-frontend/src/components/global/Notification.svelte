<!-- https://dev.to/danawoodman/svelte-quick-tip-creating-a-toast-notification-system-ge3 -->

<script lang="ts">
  import { fade } from "svelte/transition";
  import type { ToastType } from "./toaster.svelte";
  let {
    type,
    message,
    onclose,
  }: { type: ToastType; message: string; onclose: () => void } = $props();
</script>

<!-- TODO: button to close -->

<article
  class={type}
  style="display:flex; flex-direction:row"
  role="alert"
  transition:fade
>
  <div
    style="flex-grow: 2;
    text-align: center;"
    class="text"
  >
    {message}
  </div>
  <button title="close-notification" onclick={onclose}>x</button>
</article>

<style lang="postcss">
  article {
    color: var(--pico-ontrast);
    padding: 0.75rem 1.5rem;
    border-radius: 0.2rem;
    display: flex;
    align-items: center;
    margin: 0 auto 0.5rem auto;
    width: 20rem;
  }
  /* .failure {
    background: var(--pico-del-color);
  }
  .success {
    background: var(--pico-ins-color);
  }
  .info {
    background: var(--pico-secondary-background);
  } */
  .text {
    margin-left: 1rem;
  }
  button {
    color: white;
    background: transparent;
    border: 0 none;
    padding: 0;
    margin: 0 0 0 auto;
    line-height: 1;
    font-size: 1rem;
  }
</style>
