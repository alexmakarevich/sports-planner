import * as dotenv from "dotenv";
import z = require("zod");

const envSchema = z.object({
  BACKEND_URL: z.string(),
  CONDUCTOR_USERNAME: z.string(),
  CONDUCTOR_PASSWORD: z.string(),
});

const rawConfig = dotenv.config();

const { BACKEND_URL, CONDUCTOR_PASSWORD, CONDUCTOR_USERNAME } = envSchema.parse(
  rawConfig.parsed
);

const API_URL = BACKEND_URL + "/api";

export { BACKEND_URL, CONDUCTOR_PASSWORD, CONDUCTOR_USERNAME, API_URL };
