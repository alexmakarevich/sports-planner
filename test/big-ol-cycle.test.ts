import { logIn, makeTestId, signUpViaInvite, signUpWithNewOrg } from "./utils";
import { Client } from "./utils/client";
import { randomUUID } from "crypto";
import { AxiosError, AxiosResponse } from "axios";

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

    const org_adminClient = new Client({
      ...neworg_adminDetails,
      isTest: true,
    });

    const ownRolesOfAdmin = await org_adminClient.listOwnRoles();
    expect(ownRolesOfAdmin).toEqual(["org_admin"]);

    const allRoleAssignments = await org_adminClient.listRoles();
    expect(allRoleAssignments).toEqual({
      [org_adminClient.ownId]: ["org_admin"],
    });

    const newUserId = await org_adminClient.createUser({
      username: regularUserName,
      password: regularUserPassword,
    });

    const regularUserDetails = await logIn({
      username: regularUserName,
      password: regularUserPassword,
    });

    const regularUserClient = new Client({
      ...regularUserDetails,
      isTest: true,
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual([]);

    await expect(
      regularUserClient.createUser({
        username: "test-delete-pls-" + randomUUID(),
        password: "cckwmckwekcrk",
      })
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    await expect(
      regularUserClient.deleteUserById("jdjdjjd")
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    const users = await org_adminClient.listUsers();
    expect(users.length).toEqual(2);
    expect(users).toEqual(
      expect.arrayContaining([
        {
          id: org_adminClient.ownId,
          username: adminUsername,
        },
        {
          id: regularUserClient.ownId,
          username: regularUserName,
        },
      ])
    );

    await org_adminClient.assignRole({
      user_id: regularUserClient.ownId,
      role: "org_admin",
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual(["org_admin"]);

    expect(org_adminClient.listRoles()).resolves.toEqual({
      [org_adminClient.ownId]: ["org_admin"],
      [regularUserClient.ownId]: ["org_admin"],
    });

    const id = await regularUserClient.createUser({
      username: "test-delete-pls-" + randomUUID(),
      password: "cckwmckwekcrk",
    });
    await regularUserClient.deleteUserById(id);

    await org_adminClient.unassignRole({
      user_id: regularUserClient.ownId,
      role: "org_admin",
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual([]);
    expect(org_adminClient.listRoles()).resolves.toMatchObject({
      [org_adminClient.ownId]: ["org_admin"],
      // [regularUserClient.ownId]: NONE,
    });

    await expect(
      regularUserClient.createUser({
        username: "test-delete-pls-" + randomUUID(),
        password: "cckwmckwekcrk",
      })
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    await expect(
      regularUserClient.deleteUserById("jdjdjjd")
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [org_admin, super_admin]",
      },
    });

    //

    const serviceInviteId = await org_adminClient.createServiceInvite();

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

    // Failed to deserialize the JSON body into the target type: my_type: unknown variant `orange`, expected one of `Orange`, `Apple`

    // creating game

    await org_adminClient.assignRole({
      user_id: regularUserClient.ownId,
      role: "coach",
    });

    await org_adminClient.assignRole({
      user_id: regularUserClient.ownId,
      role: "player",
    });

    const newGgameId = await org_adminClient.createGame({
      opponent: "some-opp",
      start: new Date(),
      end: new Date(),
      location: "some place with address",
      location_kind: "home",
      invited_roles: ["player", "coach"],
    });

    await org_adminClient.deleteServiceInviteById(serviceInviteId);

    await invitedClient.deleteOwnUser();

    // NORMAL CLEANUP

    await org_adminClient.deleteUserById(newUserId);

    await org_adminClient.deleteOwnOrg();
  });
});
