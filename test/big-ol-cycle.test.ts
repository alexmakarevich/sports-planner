import { logIn, makeTestId, signUpViaInvite, signUpWithNewOrg } from "./utils";
import { Client } from "./utils/client";
import { randomUUID } from "crypto";
import { AxiosError, AxiosResponse } from "axios";

const { testId } = makeTestId();

const regularUserName = "regular-" + testId;
const regularUserPassword = regularUserName;

const invitedUserName = "invited-" + testId;
const invitedUserPassword = regularUserName;

// TODO: ensure cleanup
// TODO: log out

describe(__filename, () => {
  it("does it all...", async () => {
    const newOrgCookie = await signUpWithNewOrg({
      username: "admin-user-" + testId,
      password: "test-password-" + testId,
      orgTitle: "test-org-" + testId,
    });

    const orgAdminClient = new Client({ cookie: newOrgCookie, isTest: true });

    const newUserId = await orgAdminClient.createUser({
      username: regularUserName,
      password: regularUserPassword,
    });

    const regularUserCookie = await logIn({
      username: regularUserName,
      password: regularUserPassword,
    });

    const regularUserClient = new Client({
      cookie: regularUserCookie,
      isTest: true,
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

    const serviceInviteId = await orgAdminClient.createServiceInvite();

    const cookieFromInvite = await signUpViaInvite({
      username: invitedUserName,
      password: invitedUserPassword,
      inviteId: serviceInviteId,
    });

    const invitedClient = new Client({
      cookie: cookieFromInvite,
      isTest: true,
    });

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
