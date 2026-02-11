import { makeTestId } from "./utils/general";
import { TestClient } from "./utils/test-client";
import axios from "axios";
import { DateTime } from "luxon";
import { API_URL } from "./utils/env";
import { log } from "console";
import { testAuthUtils } from "./utils/auth";

const { testId } = makeTestId();

const regularUserName = "regular-" + testId;
const regularUserPassword = regularUserName;

const invitedUserName = "invited-" + testId;
const invitedUserPassword = regularUserName;

// TODO: ensure cleanup

describe(__filename, () => {
  it("denies req with no cookie", async () => {
    // @ts-expect-error
    const c = new TestClient({ cookie: undefined, testId });

    for (const method of [
      () => c.createServiceInvite(),
      () => c.createUser({ username: "", password: "" }),
      () => c.deleteOwnOrg(),
      () => c.deleteOwnUser(),
      () => c.deleteOwnUser(),
      () => c.deleteServiceInviteById("222ede"),
      () => c.deleteServiceInviteById("22cedede2"),
      () => c.listUsers(),
      () => c.deleteUserById("dsede"),
      () => c.logOut(),
    ]) {
      await expect(method()).rejects.toMatchObject({
        response: {
          status: 401,
          data: "Not logged in",
        },
      });
    }
  });

  it.only("performs cookie lifecycle", async () => {
    const { cookie: conductorCookie, ownId } =
      await testAuthUtils.logInConductorUser();
    console.log(conductorCookie);
    expect(conductorCookie.slice(0, 11)).toEqual("session_id=");
    expect(conductorCookie.slice(11, 27)).not.toMatch(";");
    expect(conductorCookie.slice(27, 72)).toEqual(
      "; HttpOnly; SameSite=Strict; Secure; Expires=",
    );
    const expirationString = conductorCookie.slice(72);

    const expirationDate = DateTime.fromRFC2822(expirationString);

    if (!expirationDate.isValid) {
      throw new Error("invalid date");
    }

    const diff = expirationDate.diffNow("day", {}).days;

    expect(Math.round(diff)).toEqual(7);

    const client = new TestClient({ cookie: conductorCookie, ownId, testId });
    await client.logOut();

    const { status, data, headers } = await axios({
      url: API_URL + "/users/list",
      headers: {
        Cookie: conductorCookie,
      },
      validateStatus: () => true,
    });

    log({ status, data, headers });

    expect(status).toEqual(401);
    expect(data).toEqual("Unauthorized");

    const cookies = headers["set-cookie"];

    const unsetSessionCookie = cookies?.find((c) =>
      c.startsWith("session_id="),
    );

    if (!unsetSessionCookie) {
      throw new Error("no unset cookie returned");
    }

    expect(unsetSessionCookie.slice(0, 56)).toEqual(
      "session_id=; HttpOnly; SameSite=Strict; Secure; Expires=",
    );

    const expirationStringOfUnsetCookie = unsetSessionCookie.slice(57);

    const expirationDateOfUnsetCookie = new Date(
      expirationStringOfUnsetCookie,
    ).valueOf();

    expect(expirationDateOfUnsetCookie).toEqual(0);
  });
});
