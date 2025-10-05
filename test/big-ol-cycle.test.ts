import { logIn, logInCoductorUser, signUpWithNewOrg } from "./utils";
import { Client } from "./utils/client";
import { DateTime } from "luxon";
import util from "util";
import { randomUUID } from "crypto";
import path from "path";
import { AxiosError, AxiosResponse } from "axios";

const timestamp = DateTime.now().toFormat("yyMMdd-HHmmss");
const callSites = util.getCallSites();
const filePath = callSites[0].scriptName;
const fileName = path.parse(filePath).base;
const testId = timestamp + randomUUID().slice(0, 4) + fileName;

const regularUserName = "regular-" + testId;
const regularUserPassword = regularUserName;

// TODO: ensure cleanup
// TODO: log out

describe.only(__filename, () => {
  it("does it all...", async () => {
    const conductorCookie = await logInCoductorUser();

    const newOrgCookie = await signUpWithNewOrg({
      username: "test-user-" + testId,
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

    let errorResponse: AxiosResponse | undefined;

    try {
      await regularUserClient.createUser({
        username: "test-delete-pls-" + randomUUID(),
        password: "cckwmckwekcrk",
      });
    } catch (err) {
      if (err instanceof AxiosError) {
        errorResponse = err.response;
        console.info(err.response?.status, err.response?.data);
      } else {
        console.info(err);
      }
    }

    expect(errorResponse?.status).toEqual(401);
    expect(errorResponse?.data).toEqual(
      "Access denied. Needs one of roles: [OrgAdmin, SuperAdmin]"
    );

    try {
      await regularUserClient.deleteUserById("jdjdjjd");
    } catch (err) {
      if (err instanceof AxiosError) {
        errorResponse = err.response;
        console.info(err.response?.status, err.response?.data);
      } else {
        console.info(err);
      }
    }

    expect(errorResponse?.status).toEqual(401);
    expect(errorResponse?.data).toEqual(
      "Access denied. Needs one of roles: [OrgAdmin, SuperAdmin]"
    );

    await orgAdminClient.deleteUserById(newUserId);

    await orgAdminClient.deleteOwnOrg();
  });
});
