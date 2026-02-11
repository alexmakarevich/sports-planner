import * as dotenv from "dotenv";
import z from "zod";

// const envSchema = z.object({
//   BACKEND_URL: z.string(),
// });

// const rawConfig = dotenv.config();

// const { BACKEND_URL } = envSchema.parse(rawConfig.parsed);

const BACKEND_URL = "http://localhost:3333";

const API_URL = BACKEND_URL + "/api";

export { BACKEND_URL, API_URL };
