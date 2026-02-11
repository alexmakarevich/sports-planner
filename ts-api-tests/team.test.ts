import { testAuthUtils } from "./utils/auth";
import { TestClient } from "./utils/test-client";
import { makeTestId } from "./utils/general";

const { testId } = makeTestId();

describe(__filename, () => {
  it("CRUD team entity (admin) and permission checks (regular user)", async () => {
    // Create a brand‑new club – this gives us an club‑admin user
    const adminDetails = await testAuthUtils.signUpWithNewClub({
      username: `admin-${testId}`,
      password: `admin-pass-${testId}`,
      clubTitle: `test-club-${testId}`,
    });

    const clubAdminClient = new TestClient({ ...adminDetails, testId });

    // ---- Admin can create a team ---------------------------------------
    const teamName = `team-${testId}`;
    const teamSlug = `slug-${testId}`;

    const teamId = await clubAdminClient.createTeam({
      name: teamName,
      slug: teamSlug,
    });

    // ---- List teams ----------------------------------------------------
    const teamsAfterCreate = await clubAdminClient.listTeams();
    expect(teamsAfterCreate).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: teamId,
          name: teamName,
          slug: teamSlug,
        }),
      ]),
    );

    // ---- Get a single team ---------------------------------------------
    const teamDetails = await clubAdminClient.getTeam(teamId);
    expect(teamDetails).toEqual(
      expect.objectContaining({
        id: teamId,
        name: teamName,
        slug: teamSlug,
      }),
    );

    // ---- Update the team -----------------------------------------------
    const newName = `${teamName}-updated`;
    const newSlug = `${teamSlug}-updated`;

    const updatedTeam = await clubAdminClient.updateTeam(teamId, {
      name: newName,
      slug: newSlug,
    });

    expect(updatedTeam).toEqual(
      expect.objectContaining({
        id: teamId,
        name: newName,
        slug: newSlug,
      }),
    );

    // ---- Delete the team ------------------------------------------------
    await clubAdminClient.deleteTeamById(teamId);

    // Team should no longer appear in the list
    const teamsAfterDelete = await clubAdminClient.listTeams();
    expect(teamsAfterDelete).not.toEqual(
      expect.arrayContaining([expect.objectContaining({ id: teamId })]),
    );

    // ---- Regular user cannot create teams ------------------------------
    // Create a second user (no special roles yet)
    const userId = await clubAdminClient.createUser({
      username: `user-${testId}`,
      password: `user-pass-${testId}`,
    });

    const userLogin = await testAuthUtils.logIn({
      username: `user-${testId}`,
      password: `user-pass-${testId}`,
    });

    const userClient = new TestClient({ ...userLogin, testId });

    await expect(
      userClient.createTeam({
        name: `bad-team-${testId}`,
        slug: `bad-slug-${testId}`,
      }),
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [super_admin, club_admin]",
      },
    });

    // Clean up user
    await userClient.deleteOwnUser();

    // ---- Cleanup: delete the club ----------------------------------------
    await clubAdminClient.deleteOwnclub();
  });
});
