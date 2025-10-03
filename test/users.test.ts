import axios from "axios";
import { logIn, logInCoductorUser } from "./utils";
import { API_URL } from "./utils/env";

let testCookie: string;

describe(__filename, () => {
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
