import { AuthUtils } from "ts-shared";
import { API_URL } from "../utils/env";

export const authUtils = new AuthUtils({ API_URL, kind: "browser" });
