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

describe.only(__filename, () => {
  it("denies req with no cookie", async () => {
    // @ts-expect-error
    const c = new Client({ cookie: undefined });

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
});
