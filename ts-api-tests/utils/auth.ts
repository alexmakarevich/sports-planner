import axios, { AxiosRequestConfig, AxiosPromise } from "axios";
import { AuthUtils } from "ts-shared";
import { API_URL, CONDUCTOR_PASSWORD, CONDUCTOR_USERNAME } from "./env";
import { makeTestAxios } from "./general";

class TestAuthUtils extends AuthUtils {
  constructor() {
    super({ API_URL, kind: "node" });
  }
  axios: (x: AxiosRequestConfig) => AxiosPromise = makeTestAxios(
    axios.create(),
  );

  logInConductorUser() {
    return this.logIn({
      username: CONDUCTOR_USERNAME,
      password: CONDUCTOR_PASSWORD,
    });
  }
}

export const testAuthUtils = new TestAuthUtils();
