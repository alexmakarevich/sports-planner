import axios, { AxiosInstance, AxiosRequestConfig } from "axios";
import { API_URL, CONDUCTOR_PASSWORD, CONDUCTOR_USERNAME } from "./utils/env";
import { DateTime } from "luxon";
import util from "util";
import { randomUUID } from "crypto";
import path from "path";
import z from "zod";

const loginResSchema = z.string();

export type LoginResult = { ownId: string; cookie: string };

export const logIn = async ({
  username,
  password,
}: {
  username: string;
  password: string;
}): Promise<LoginResult> => {
  const { status, data, headers } = await axios({
    method: "POST",
    url: API_URL + "/log-in",
    data: {
      username,
      password,
    },
    validateStatus: () => true,
  });

  const ownId = loginResSchema.parse(data);

  const cookies = headers["set-cookie"];
  if (Array.isArray(cookies)) {
    const cookie = cookies.find((c) => c.startsWith("session_id="));
    if (cookie) {
      return { ownId, cookie };
    }
  }
  throw new Error("Failed to retieve cookie from login");
};

export const logInConductorUser = () =>
  logIn({
    username: CONDUCTOR_USERNAME,
    password: CONDUCTOR_PASSWORD,
  });

export const signUpWithNewOrg = async ({
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
    url: API_URL + "/sign-up-with-new-org",
    data: {
      username,
      password,
      org_title: orgTitle,
    },
    validateStatus: () => true,
  });
  console.log({ status, data, headers });

  const ownId = loginResSchema.parse(data);

  const cookies = headers["set-cookie"];
  if (Array.isArray(cookies)) {
    const cookie = cookies.find((c) => c.startsWith("session_id="));
    if (cookie) {
      return { ownId, cookie };
    }
  }
  throw new Error("Failed to retieve cookie from signup with new org");
};

export const signUpViaInvite = async ({
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
    url: API_URL + "/sign-up-via-invite/" + inviteId,
    data: {
      username,
      password,
    },
    validateStatus: () => true,
  });
  console.log({ status, data, headers });

  const ownId = loginResSchema.parse(data);

  const cookies = headers["set-cookie"];
  if (Array.isArray(cookies)) {
    const cookie = cookies.find((c) => c.startsWith("session_id="));
    if (cookie) {
      return { ownId, cookie };
    }
  }
  throw new Error("Failed to retieve cookie from signup with new org");
};

export const makeTestAxios = (axiosInstance: AxiosInstance) => {
  return async (reqParams: AxiosRequestConfig) => {
    try {
      console.log("test axios called");
      return await axiosInstance(reqParams);
    } catch (err) {
      const { method, url, params, data } = reqParams;
      console.warn(
        "Error in test Axios [may be expected]",
        { method, url, params, data },
        err,
      );
      throw err;
    }
  };
};

export const makeTestId = () => {
  const timestamp = DateTime.now().toFormat("yyMMdd-HHmmss");
  const callSites = util.getCallSites();
  const filePath = callSites[0].scriptName;
  const fileName = path.parse(filePath).base;
  const testId = timestamp + randomUUID().slice(0, 4) + fileName;
  return { testId };
};
