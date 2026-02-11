<script lang="ts">
  import { Link } from "svelte-routing";
  import { globalToaster } from "./global/toaster.svelte";
  import { frontendClient } from "../client/fe-client";
  let { children } = $props();
</script>

<header>
  <nav class="main-menu">
    <Link to="">home</Link>
    <Link to="users">users</Link>
    <Link to="teams">teams</Link>
    <Link to="events">events</Link>
    <!-- <Link to="my-invites">my invites</Link> -->
    <!-- <Link to="settings">settngs</Link> -->

    <Link
      to="login"
      onclick={async () => {
        try {
          await frontendClient.logOut();
          globalToaster.add({ message: "logged out" });
        } catch (err) {
          console.error(err);
          globalToaster.add({ message: "failed to log out", type: "failure" });
        }
      }}>log out</Link
    >
  </nav>
</header>
<main class="container">
  {@render children?.()}
</main>
