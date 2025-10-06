import axios, { AxiosInstance, AxiosRequestConfig } from "axios";
import { API_URL, CONDUCTOR_PASSWORD, CONDUCTOR_USERNAME } from "./utils/env";

export const logIn = async ({
  username,
  password,
}: {
  username: string;
  password: string;
}): Promise<string> => {
  const { status, data, headers } = await axios({
    method: "POST",
    url: API_URL + "/log-in",
    data: {
      username,
      password,
    },
    validateStatus: () => true,
  });

  console.log({ status, data, headers });
  const cookies = headers["set-cookie"];
  if (Array.isArray(cookies)) {
    const specificCookie = cookies.find((c) => c.startsWith("session_id="));
    if (specificCookie) {
      return specificCookie;
    }
  }
  throw new Error("Failed to retieve cookie from login");
};

export const logInCoductorUser = () =>
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
}): Promise<string> => {
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

  const cookies = headers["set-cookie"];
  if (Array.isArray(cookies)) {
    const specificCookie = cookies.find((c) => c.startsWith("session_id="));
    if (specificCookie) {
      return specificCookie;
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
}): Promise<string> => {
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

  const cookies = headers["set-cookie"];
  if (Array.isArray(cookies)) {
    const specificCookie = cookies.find((c) => c.startsWith("session_id="));
    if (specificCookie) {
      return specificCookie;
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
      console.error("Error in test Axios", { method, url, params, data }, err);
      throw err;
    }
  };
};
