import axios from "axios";
import { makeTestAxios } from "./utils/general";
import { API_URL } from "./utils/env";
import { testAuthUtils } from "./utils/auth";

let testCookie: string;

describe(__filename, () => {
  // it.only("logs in", async () => {
  //   const cookie = await logInCoductorUser();
  // });
  beforeAll(async () => {
    testCookie = (await testAuthUtils.logInConductorUser()).cookie;
  });

  it("gets list of users", async () => {
    console.log({ testCookie });

    const testAxios = makeTestAxios(axios);

    const { status, data } = await testAxios({
      url: API_URL + "/users/list",
      headers: {
        Cookie: testCookie,
      },
    });

    console.log({ status, data });
  });
});
