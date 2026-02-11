import { AuthUtils } from "ts-shared";
import { API_URL, CONDUCTOR_PASSWORD, CONDUCTOR_USERNAME } from "./env";

class TestAuthUtils extends AuthUtils {
  constructor() {
    super({ API_URL, kind: "node" });
  }

  logInConductorUser() {
    return this.logIn({
      username: CONDUCTOR_USERNAME,
      password: CONDUCTOR_PASSWORD,
    });
  }
}

export const testAuthUtils = new TestAuthUtils();
