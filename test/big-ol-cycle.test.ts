import { logIn, makeTestId, signUpViaInvite, signUpWithNewOrg } from "./utils";
import { Client } from "./utils/client";
import { randomUUID } from "crypto";

const { testId } = makeTestId();

const adminUsername = "admin-user-" + testId;
const adminPassword = adminUsername;

const regularUserName = "regular-" + testId;
const regularUserPassword = regularUserName;

const invitedUserName = "invited-" + testId;
const invitedUserPassword = regularUserName;

// TODO: ensure cleanup
// TODO: log out

describe(__filename, () => {
  it("does it all...", async () => {
    const neworg_adminDetails = await signUpWithNewOrg({
      username: adminUsername,
      password: adminPassword,
      orgTitle: "test-org-" + testId,
    });

    const orgAdminClient = new Client({
      ...neworg_adminDetails,
      isTest: true,
    });

    const ownRolesOfAdmin = await orgAdminClient.listOwnRoles();
    expect(ownRolesOfAdmin).toEqual(["org_admin"]);

    const allRoleAssignments = await orgAdminClient.listRoles();
    expect(allRoleAssignments).toEqual({
      [orgAdminClient.ownId]: ["org_admin"],
    });

    const newUserId = await orgAdminClient.createUser({
      username: regularUserName,
      password: regularUserPassword,
    });

    let regularUserDetails = await logIn({
      username: regularUserName,
      password: regularUserPassword,
    });

    let regularUserClient = new Client({
      ...regularUserDetails,
      isTest: true,
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual([]);

    await expect(
      regularUserClient.createUser({
        username: "test-delete-pls-" + randomUUID(),
        password: "cckwmckwekcrk",
      }),
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    await expect(
      regularUserClient.deleteUserById("jdjdjjd"),
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    const users = await orgAdminClient.listUsers();
    expect(users.length).toEqual(2);
    expect(users).toEqual(
      expect.arrayContaining([
        {
          id: orgAdminClient.ownId,
          username: adminUsername,
        },
        {
          id: regularUserClient.ownId,
          username: regularUserName,
        },
      ]),
    );

    await orgAdminClient.assignRole({
      user_id: regularUserClient.ownId,
      role: "org_admin",
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual(["org_admin"]);

    expect(orgAdminClient.listRoles()).resolves.toEqual({
      [orgAdminClient.ownId]: ["org_admin"],
      [regularUserClient.ownId]: ["org_admin"],
    });

    const id = await regularUserClient.createUser({
      username: "test-delete-pls-" + randomUUID(),
      password: "cckwmckwekcrk",
    });
    await regularUserClient.deleteUserById(id);

    await orgAdminClient.unassignRole({
      user_id: regularUserClient.ownId,
      role: "org_admin",
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual([]);
    expect(orgAdminClient.listRoles()).resolves.toMatchObject({
      [orgAdminClient.ownId]: ["org_admin"],
      // [regularUserClient.ownId]: NONE,
    });

    await expect(
      regularUserClient.createUser({
        username: "test-delete-pls-" + randomUUID(),
        password: "cckwmckwekcrk",
      }),
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    await expect(
      regularUserClient.deleteUserById("jdjdjjd"),
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    //

    const serviceInviteId = await orgAdminClient.createServiceInvite();

    const inviteUserDetails = await signUpViaInvite({
      username: invitedUserName,
      password: invitedUserPassword,
      inviteId: serviceInviteId,
    });

    const invitedClient = new Client(inviteUserDetails);

    const usersListed = await invitedClient.listUsers();
    console.log({ usersListed });

    await regularUserClient.logOut();

    await expect(regularUserClient.listUsers()).rejects.toMatchObject({
      response: {
        status: 401,
        data: "Unauthorized",
      },
    });

    // creating game

    await orgAdminClient.assignRole({
      user_id: regularUserClient.ownId,
      role: "coach",
    });

    await orgAdminClient.assignRole({
      user_id: regularUserClient.ownId,
      role: "player",
    });

    // re-login after logout
    regularUserDetails = await logIn({
      username: regularUserName,
      password: regularUserPassword,
    });

    regularUserClient = new Client({
      ...regularUserDetails,
      isTest: true,
    });

    let regularUserInvites = await regularUserClient.listOwnInvites();

    expect(regularUserInvites).toEqual([]);

    // create team

    const teamName = `team-${testId}`;
    const teamSlug = `slug-${testId}`;

    const team_id = await orgAdminClient.createTeam({
      name: teamName,
      slug: teamSlug,
    });

    const newGameId = await orgAdminClient.createGame({
      team_id,
      opponent: "some-opp",
      start: new Date(),
      end: new Date(),
      location: "some place with address",
      location_kind: "home",
      invited_roles: ["player", "coach"],
    });

    regularUserInvites = await regularUserClient.listOwnInvites();
    expect(regularUserInvites.length).toEqual(1);
    expect(regularUserInvites).toMatchObject([
      {
        game_id: newGameId,
        opponent: "some-opp",
        response: "pending",
      },
    ]);
    const firstInviteId = regularUserInvites[0].invite_id;

    const invitesToFirstGame =
      await orgAdminClient.listInvitesToGame(newGameId);

    expect(invitesToFirstGame).toEqual([
      {
        invite_id: firstInviteId,
        user_id: regularUserClient.ownId,
        username: regularUserName,
        response: "pending",
      },
    ]);

    await regularUserClient.respondToInvite({
      invite_id: firstInviteId,
      response: "unsure",
    });

    await expect(orgAdminClient.listInvitesToGame(newGameId)).resolves.toEqual([
      {
        invite_id: firstInviteId,
        user_id: regularUserClient.ownId,
        username: regularUserName,
        response: "unsure",
      },
    ]);

    await regularUserClient.respondToInvite({
      invite_id: firstInviteId,
      response: "declined",
    });

    await expect(orgAdminClient.listInvitesToGame(newGameId)).resolves.toEqual([
      {
        invite_id: firstInviteId,
        user_id: regularUserClient.ownId,
        username: regularUserName,
        response: "declined",
      },
    ]);

    await regularUserClient.respondToInvite({
      invite_id: firstInviteId,
      response: "accepted",
    });

    await expect(orgAdminClient.listInvitesToGame(newGameId)).resolves.toEqual([
      {
        invite_id: firstInviteId,
        user_id: regularUserClient.ownId,
        username: regularUserName,
        response: "accepted",
      },
    ]);

    // await expect(
    //   regularUserClient.respondToInvite({
    //     invite_id: firstInviteId,
    //     // @ts-expect-error
    //     response: "pending",
    //   }),
    // ).rejects.toMatchObject({
    //   response: {
    //     status: 422,
    //     data: /"Failed to deserialize the JSON body into the target type: response: unknown variant `something-else`, expected one of `accepted`, `declined`, `unsure`/,
    //   },
    // });

    // await expect(
    //   regularUserClient.respondToInvite({
    //     invite_id: firstInviteId,
    //     // @ts-expect-error
    //     response: "something-else",
    //   }),
    // ).rejects.toMatchObject({
    //   response: {
    //     status: 422,
    //     data: /"Failed to deserialize the JSON body into the target type: response: unknown variant `something-else`, expected one of `accepted`, `declined`, `unsure`/,
    //   },
    // });

    // TODO: GAME CLEANUP!

    // SERVICE INVITE CLEANUP

    await orgAdminClient.deleteServiceInviteById(serviceInviteId);

    await invitedClient.deleteOwnUser();

    // NORMAL CLEANUP

    await orgAdminClient.deleteUserById(newUserId);

    await orgAdminClient.deleteOwnOrg();
  });
});
