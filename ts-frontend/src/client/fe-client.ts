import { Client } from "ts-shared";
import { API_URL } from "../utils/env";

export const frontendClient = new Client({
  API_URL,
  cookie: "unused-cookie",
  kind: "browser",
});
