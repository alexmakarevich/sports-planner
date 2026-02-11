<script lang="ts">
  import { onMount } from "svelte";
  import { frontendClient } from "../../client/fe-client";
  import type { Team } from "ts-shared";

  let teams: Team[] = [];
  let loading = true;
  let error: string | null = null;

  // Simulate API call - in real app this would call your service
  async function fetchTeams() {
    try {
      loading = true;
      error = null;

      const teams = await frontendClient.listTeams();
    } catch (err) {
      error = "Failed to load teams";
      console.error(err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchTeams();
  });
</script>

<main class="team-list-page">
  <header>
    <h1>Teams</h1>
  </header>

  {#if loading}
    <p>Loading teams...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if teams.length === 0}
    <p>No teams found</p>
  {:else}
    <section>
      <ul class="teams-list">
        {#each teams as team (team.id)}
          <li class="team-item">
            <h2>{team.name}</h2>
            <p>Slug: {team.slug}</p>
            <p>Organization ID: {team.org_id}</p>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
</main>
