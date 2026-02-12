import { Client } from "ts-shared";
import { API_URL } from "../utils/env";

export const frontendClient = new Client({
  API_URL,
  cookie: "unused-cookie",
  kind: "browser",
  // isGlobalAdmin: true,
});
// TODO: set isAdmin, after logging in through admin login form?
// or make it simpler and more generic???
