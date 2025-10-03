import axios from "axios";
import {
  BACKEND_URL,
  CONDUCTOR_PASSWORD,
  CONDUCTOR_USERNAME,
} from "./utils/env";

export const logIn = async ({
  username,
  password,
}: {
  username: string;
  password: string;
}): Promise<string> => {
  const { status, data, headers } = await axios({
    method: "POST",
    url: BACKEND_URL + "/login",
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
