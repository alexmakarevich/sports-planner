import axios from "axios";
import { logInConductorUser, makeTestAxios } from "./utils";
import { API_URL } from "./utils/env";

let testCookie: string;

describe(__filename, () => {
  // it.only("logs in", async () => {
  //   const cookie = await logInCoductorUser();
  // });
  beforeAll(async () => {
    testCookie = (await logInConductorUser()).cookie;
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
