<script lang="ts">
  import { navigate } from "svelte-routing";
  import { globalToaster } from "./global/toaster.svelte";
  import { authUtils } from "../client/auth";

  let username = $state("");
  let password = $state("");
</script>

<!-- TODO: handle case where you're already logged in: -->
<!-- "you're already logged in, log out? go back?" *fields disabled* -->

<h2>log in</h2>

<input type="text" bind:value={username} />
<input type="password" bind:value={password} />

<button
  onclick={async () => {
    try {
      await authUtils.logIn({
        username,
        password,
      });
      // TODO: allow navigating to previously attempted pages via query params
      navigate("/");
      globalToaster.add({ type: "success", message: "logged in!" });
    } catch (err) {
      console.error(err);
      globalToaster.add({ type: "failure", message: "login failed" });
    }
  }}>log in</button
>
