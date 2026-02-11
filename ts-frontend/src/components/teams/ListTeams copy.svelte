<script lang="ts">
  import { onMount } from "svelte";
  import { frontendClient } from "../../client/fe-client";
  import type { Team } from "ts-shared";

  let teams: Team[] = [];
  let loading = true;
  let error: string | null = null;
  let showForm = false;
  let formLoading = false;
  let formError: string | null = null;
  let formData = { name: "", slug: "" };

  onMount(async () => {
    await loadTeams();
  });

  async function loadTeams() {
    try {
      teams = await frontendClient.listTeams();
      error = null;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load teams";
    } finally {
      loading = false;
    }
  }

  async function handleCreateTeam(e: SubmitEvent) {
    e.preventDefault();
    formLoading = true;
    formError = null;

    try {
      await frontendClient.createTeam({
        name: formData.name,
        slug: formData.slug,
      });
      formData = { name: "", slug: "" };
      showForm = false;
      await loadTeams();
    } catch (err) {
      formError = err instanceof Error ? err.message : "Failed to create team";
    } finally {
      formLoading = false;
    }
  }

  function toggleForm() {
    showForm = !showForm;
    if (!showForm) {
      formError = null;
      formData = { name: "", slug: "" };
    }
  }
</script>

<main>
  <h1>Teams</h1>

  <button on:click={toggleForm}>
    {showForm ? "Cancel" : "Create Team"}
  </button>

  {#if showForm}
    <form on:submit={handleCreateTeam}>
      <fieldset>
        <legend>New Team</legend>

        {#if formError}
          <p role="alert">{formError}</p>
        {/if}

        <div>
          <label for="team-name">Name</label>
          <input
            id="team-name"
            type="text"
            name="name"
            bind:value={formData.name}
            required
            disabled={formLoading}
          />
        </div>

        <div>
          <label for="team-slug">Slug</label>
          <input
            id="team-slug"
            type="text"
            name="slug"
            bind:value={formData.slug}
            required
            disabled={formLoading}
          />
        </div>

        <button type="submit" disabled={formLoading}>
          {formLoading ? "Creating..." : "Create"}
        </button>
      </fieldset>
    </form>
  {/if}

  {#if loading}
    <p>Loading teams...</p>
  {:else if error}
    <p role="alert">{error}</p>
  {:else if teams.length === 0}
    <p>No teams found.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Slug</th>
          <th>Organization ID</th>
        </tr>
      </thead>
      <tbody>
        {#each teams as team (team.id)}
          <tr>
            <td>{team.name}</td>
            <td>{team.slug}</td>
            <td>{team.org_id}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</main>
