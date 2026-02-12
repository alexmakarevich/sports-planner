import { makeTestId } from "./utils/general";
import { testAuthUtils } from "./utils/auth";
import { TestClient } from "./utils/test-client";

let conductorClient: TestClient;
const { testId } = makeTestId();

describe(__filename, () => {
  beforeAll(async () => {
    const { cookie, ownId } = await testAuthUtils.logInConductorUser();
    conductorClient = new TestClient({
      cookie,
      ownId,
      testId,
    });
  });

  afterAll(async () => {
    await conductorClient.logOut();
  });

  it("gets list of users", async () => {
    const users = await conductorClient.listUsers();
    console.log({ users });
  });
});
