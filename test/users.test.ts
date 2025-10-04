import axios from "axios";
import { logInCoductorUser } from "./utils";
import { API_URL } from "./utils/env";

let testCookie: string;

describe.skip(__filename, () => {
  // it.only("logs in", async () => {
  //   const cookie = await logInCoductorUser();
  // });
  beforeAll(async () => {
    testCookie = await logInCoductorUser();
  });

  it("gets list of users", async () => {
    console.log({ testCookie });

    const { status, data } = await axios({
      url: API_URL + "/users/list",
      headers: {
        Cookie: testCookie,
      },
    });

    console.log({ status, data });
  });
});
