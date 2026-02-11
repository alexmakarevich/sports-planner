import { isAxiosError } from "axios";
import { API_URL } from "./env";
import { log } from "console";
import { Client } from "ts-shared";

export class TestClient extends Client {
  ownId: string;
  constructor({
    cookie,
    ownId,
    testId,
    isGlobalAdmin,
  }: {
    cookie: string;
    ownId: string;
    testId: string;
    isGlobalAdmin?: boolean;
  }) {
    super({ cookie, API_URL, kind: "node", isGlobalAdmin });
    this.ownId = ownId;

    console.log("new Test Client", { cookie });

    this.axios.interceptors.request.use((r) => {
      r.headers.set("x-test-id", testId);
      return r;
    });

    this.axios.interceptors.response.use(
      (r) => {
        return r;
      },
      (err) => {
        if (isAxiosError(err)) {
          const { response, config } = err;

          console.log(!!response, !!config);

          if (!!response && !!config) {
            const { data: reqData, method, url, params } = config;
            const { status, statusText, data: resData } = response;

            console.warn("Error in test Axios [may be expected]", {
              req: { method, url, params, data: reqData },
              res: {
                status,
                statusText,
                data: resData,
              },
            });
          } else {
            console.warn(
              "Incomplete error in test Axios [may be expected]",
              err,
            );
          }
        } else {
          console.warn("Unexpected error in test Axios [may be expected]", err);
        }

        throw err;
      },
    );
  }

  // axios: (x: AxiosRequestConfig) => AxiosPromise = makeTestAxios(
  //   axios.create({ headers: { Cookie: this.cookie } }),
  // );
}
