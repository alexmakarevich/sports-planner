<!-- https://dev.to/danawoodman/svelte-quick-tip-creating-a-toast-notification-system-ge3 -->

<script lang="ts">
  import Notification from "./Notification.svelte";

  import { globalToaster } from "./toaster.svelte";
</script>

<section>
  {#each globalToaster.toasts as toast (toast.id)}
    <Notification
      type={toast.type}
      message={toast.message}
      onclose={() => globalToaster.remove(toast.id)}
    ></Notification>
  {/each}
</section>

<style lang="postcss">
  section {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    width: 100%;
    display: flex;
    margin-top: 1rem;
    justify-content: center;
    flex-direction: column;
    z-index: 1000;
  }
</style>
