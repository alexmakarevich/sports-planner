import axios, { AxiosPromise, AxiosRequestConfig } from "axios";
import z from "zod";
import { ClientKind } from "./client";

const loginResSchema = z.string();

export type LoginResult = { ownId: string; cookie: string };

export class AuthUtils {
  axios: (x: AxiosRequestConfig) => AxiosPromise;
  API_URL: string;
  kind: ClientKind;
  constructor({
    axiosOverride,
    API_URL,
    kind,
  }: {
    axiosOverride?: (x: AxiosRequestConfig) => AxiosPromise;
    API_URL: string;
    kind: ClientKind;
  }) {
    this.axios = axiosOverride ?? axios.create();
    this.API_URL = API_URL;
    this.kind = kind;
  }

  logIn = async ({
    username,
    password,
  }: {
    username: string;
    password: string;
  }): Promise<LoginResult> => {
    const { status, data, headers } = await axios({
      method: "POST",
      url: this.API_URL + "/log-in",
      data: {
        username,
        password,
      },
      validateStatus: () => true,
    });

    const ownId = loginResSchema.parse(data);

    if (this.kind === "browser") {
      // TODO: less hacky
      return { ownId, cookie: "fake-cookie" };
    }

    const cookies = headers["set-cookie"];
    if (Array.isArray(cookies)) {
      const cookie = cookies.find((c) => c.startsWith("session_id="));
      if (cookie) {
        return { ownId, cookie };
      }
    }
    throw new Error("Failed to retieve cookie from login");
  };

  signUpWithNewOrg = async ({
    username,
    password,
    orgTitle,
  }: {
    username: string;
    password: string;
    orgTitle: string;
  }): Promise<LoginResult> => {
    const { status, data, headers } = await axios({
      method: "POST",
      url: this.API_URL + "/sign-up-with-new-org",
      data: {
        username,
        password,
        org_title: orgTitle,
      },
      validateStatus: () => true,
    });
    console.log({ status, data, headers });

    const ownId = loginResSchema.parse(data);

    if (this.kind === "browser") {
      // TODO: less hacky
      return { ownId, cookie: "fake-cookie" };
    }

    const cookies = headers["set-cookie"];
    if (Array.isArray(cookies)) {
      const cookie = cookies.find((c) => c.startsWith("session_id="));
      if (cookie) {
        return { ownId, cookie };
      }
    }
    throw new Error("Failed to retieve cookie from signup with new org");
  };

  signUpViaInvite = async ({
    username,
    password,
    inviteId,
  }: {
    username: string;
    password: string;
    inviteId: string;
  }): Promise<LoginResult> => {
    const { status, data, headers } = await axios({
      method: "POST",
      url: this.API_URL + "/sign-up-via-invite/" + inviteId,
      data: {
        username,
        password,
      },
      validateStatus: () => true,
    });
    console.log({ status, data, headers });

    const ownId = loginResSchema.parse(data);

    if (this.kind === "browser") {
      // TODO: less hacky
      return { ownId, cookie: "fake-cookie" };
    }

    const cookies = headers["set-cookie"];
    if (Array.isArray(cookies)) {
      const cookie = cookies.find((c) => c.startsWith("session_id="));
      if (cookie) {
        return { ownId, cookie };
      }
    }
    throw new Error("Failed to retieve cookie from signup with new org");
  };
}
