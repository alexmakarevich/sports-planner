import axios from "axios";
import { logInCoductorUser, signUpWithNewOrg } from "./utils";
import { API_URL } from "./utils/env";
import { Client } from "./utils/client";
import { DateTime } from "luxon";
import util from "util";
import { randomUUID } from "crypto";
import path from "path";

const timestamp = DateTime.now().toFormat("yyMMdd-HHmmss");
const callSites = util.getCallSites();
const filePath = callSites[0].scriptName;
const fileName = path.parse(filePath).base;
const testId = timestamp + randomUUID().slice(0, 4) + fileName;

describe.only(__filename, () => {
  it("does it all...", async () => {
    const conductorCookie = await logInCoductorUser();

    const newOrgCookie = await signUpWithNewOrg({
      username: "test-user-" + testId,
      password: "test-password-" + testId,
      orgTitle: "test-org-" + testId,
    });

    const orgAdminClient = new Client({ cookie: newOrgCookie, isTest: true });

    await orgAdminClient.deleteOwnOrg();
  });
});
