import { DateTime } from "luxon";
import util from "util";
import { randomUUID } from "crypto";
import path from "path";
import { AxiosInstance, AxiosRequestConfig } from "axios";

export const makeTestId = () => {
  const timestamp = DateTime.now().toFormat("yyMMdd-HHmmss");
  const callSites = util.getCallSites();
  const filePath = callSites[0].scriptName;
  const fileName = path.parse(filePath).base;
  const testId = timestamp + randomUUID().slice(0, 4) + fileName;
  return { testId };
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
