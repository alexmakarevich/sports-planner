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
    const newOrgAdminDetails = await signUpWithNewOrg({
      username: adminUsername,
      password: adminPassword,
      orgTitle: "test-org-" + testId,
    });

    const orgAdminClient = new Client({ ...newOrgAdminDetails, isTest: true });

    const ownRolesOfAdmin = await orgAdminClient.listOwnRoles();
    expect(ownRolesOfAdmin).toEqual(["OrgAdmin"]);

    const allRoleAssignments = await orgAdminClient.listRoles();
    expect(allRoleAssignments).toEqual({
      [orgAdminClient.ownId]: ["OrgAdmin"],
    });

    const newUserId = await orgAdminClient.createUser({
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
        data: "Access denied. Needs one of roles: [OrgAdmin, SuperAdmin]",
      },
    });

    await expect(
      regularUserClient.deleteUserById("jdjdjjd")
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [OrgAdmin, SuperAdmin]",
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
      ])
    );

    await orgAdminClient.assignRole({
      user_id: regularUserClient.ownId,
      role: "OrgAdmin",
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual(["OrgAdmin"]);

    expect(orgAdminClient.listRoles()).resolves.toEqual({
      [orgAdminClient.ownId]: ["OrgAdmin"],
      [regularUserClient.ownId]: ["OrgAdmin"],
    });

    const id = await regularUserClient.createUser({
      username: "test-delete-pls-" + randomUUID(),
      password: "cckwmckwekcrk",
    });
    await regularUserClient.deleteUserById(id);

    await orgAdminClient.unassignRole({
      user_id: regularUserClient.ownId,
      role: "OrgAdmin",
    });

    expect(regularUserClient.listOwnRoles()).resolves.toEqual([]);
    expect(orgAdminClient.listRoles()).resolves.toMatchObject({
      [orgAdminClient.ownId]: ["OrgAdmin"],
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
        data: "Access denied. Needs one of roles: [OrgAdmin, SuperAdmin]",
      },
    });

    await expect(
      regularUserClient.deleteUserById("jdjdjjd")
    ).rejects.toMatchObject({
      response: {
        status: 403,
        data: "Access denied. Needs one of roles: [OrgAdmin, SuperAdmin]",
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

    await orgAdminClient.deleteServiceInviteById(serviceInviteId);

    await invitedClient.deleteOwnUser();

    // NORMAL CLEANUP

    await orgAdminClient.deleteUserById(newUserId);

    await orgAdminClient.deleteOwnOrg();
  });
});
