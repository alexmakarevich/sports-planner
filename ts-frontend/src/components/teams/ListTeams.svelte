<script lang="ts">
  import { onMount } from "svelte";
  import { frontendClient } from "../../client/fe-client";
  import type { Team } from "ts-shared";
  import { globalToaster } from "../global/toaster.svelte";
  let teams: Team[] = [];
  let loading = true;
  let error: string | null = null;
  let showForm = false;
  let formLoading = false;
  let formError: string | null = null;
  let formData = { name: "", slug: "" };
  let editingTeamId: string | null = null;

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

  async function handleUpdateTeam(e: SubmitEvent) {
    e.preventDefault();
    if (!editingTeamId) return;

    formLoading = true;
    formError = null;

    try {
      await frontendClient.updateTeam(editingTeamId, {
        name: formData.name || undefined,
        slug: formData.slug || undefined,
      });
      formData = { name: "", slug: "" };
      editingTeamId = null;
      showForm = false;
      await loadTeams();
    } catch (err) {
      formError = err instanceof Error ? err.message : "Failed to update team";
    } finally {
      formLoading = false;
    }
  }

  function toggleCreateForm() {
    showForm = !showForm;
    if (!showForm) {
      formError = null;
      formData = { name: "", slug: "" };
      editingTeamId = null;
    }
  }

  function startEdit(team: Team) {
    editingTeamId = team.id;
    formData = { name: team.name, slug: team.slug };
    showForm = true;
    formError = null;
  }

  function cancelEdit() {
    editingTeamId = null;
    formData = { name: "", slug: "" };
    showForm = false;
    formError = null;
  }

  async function deleteTeam(teamId: string) {
    if (!confirm("Are you sure you want to delete this team?")) {
      return;
    }

    try {
      await frontendClient.deleteTeamById(teamId);
      await loadTeams();
      globalToaster.add({
        type: "success",
        message: "Team deleted successfully",
      });
    } catch (err) {
      globalToaster.add({
        type: "failure",
        message: "Failed to delete team",
      });
    }
  }
</script>

<main>
  <h1>Teams</h1>

  <button on:click={toggleCreateForm} disabled={showForm && !!editingTeamId}>
    {showForm && editingTeamId ? "Editing..." : "Create Team"}
  </button>

  {#if showForm}
    <form on:submit={editingTeamId ? handleUpdateTeam : handleCreateTeam}>
      <fieldset>
        <legend>{editingTeamId ? "Edit Team" : "New Team"}</legend>

        {#if formError}
          {globalToaster.add({
            type: "failure",
            message: editingTeamId
              ? "could not edit team"
              : "could not create team",
          })}
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

        <div>
          <button type="submit" disabled={formLoading}>
            {formLoading
              ? editingTeamId
                ? "Updating..."
                : "Creating..."
              : editingTeamId
                ? "Update"
                : "Create"}
          </button>
          <button type="button" on:click={cancelEdit} disabled={formLoading}>
            Cancel
          </button>
        </div>
      </fieldset>
    </form>
  {/if}

  {#if loading}
    <p>Loading teams...</p>
  {:else if error}
    {globalToaster.add({ type: "failure", message: "could not load teams" })}
  {:else if teams.length === 0}
    <p>No teams found.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Slug</th>
          <th>Club ID</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each teams as team (team.id)}
          <tr>
            <td>{team.name}</td>
            <td>{team.slug}</td>
            <td>{team.club_id}</td>
            <td>
              <button
                on:click={() => startEdit(team)}
                disabled={showForm && editingTeamId !== team.id}
              >
                Edit
              </button>
              <button
                on:click={() => deleteTeam(team.id)}
                disabled={showForm && editingTeamId !== team.id}
              >
                Delete
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</main>
