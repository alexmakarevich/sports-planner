import axios, { AxiosPromise, AxiosRequestConfig } from "axios";
import z from "zod";
import { ClientKind } from "./client";

const loginResSchema = z.string();

const AUTH_PREFIX = "/auth";

export type LoginResult = { ownId: string; cookie: string };

export class AuthUtils {
  axios: (x: AxiosRequestConfig) => AxiosPromise;
  API_URL: string;
  kind: ClientKind;
  constructor({ API_URL, kind }: { API_URL: string; kind: ClientKind }) {
    this.API_URL = API_URL;
    this.axios = axios.create({ baseURL: API_URL + AUTH_PREFIX });
    this.kind = kind;
  }

  logIn = async ({
    username,
    password,
  }: {
    username: string;
    password: string;
  }): Promise<LoginResult> => {
    const { data, headers } = await this.axios({
      method: "POST",
      url: "/log-in",
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

  signUpWithNewClub = async ({
    username,
    password,
    clubTitle,
  }: {
    username: string;
    password: string;
    clubTitle: string;
  }): Promise<LoginResult> => {
    const { status, data, headers } = await this.axios({
      method: "POST",
      url: "/sign-up-with-new-club",
      data: {
        username,
        password,
        club_title: clubTitle,
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
    throw new Error("Failed to retieve cookie from signup with new club");
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
    const { status, data, headers } = await this.axios({
      method: "POST",
      url: "/sign-up-via-invite/" + inviteId,
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
    throw new Error("Failed to retieve cookie from signup with new club");
  };
}
