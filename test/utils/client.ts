import axios, { AxiosInstance, AxiosPromise, AxiosRequestConfig } from "axios";
import { API_URL } from "./env";
import { makeTestAxios } from "../utils";

export class Client {
  cookie: string;
  axios: (x: AxiosRequestConfig) => AxiosPromise;
  constructor({ cookie, isTest }: { cookie: string; isTest?: true }) {
    console.log("new Client", { cookie, isTest });

    this.cookie = cookie;
    this.axios = isTest
      ? makeTestAxios(axios.create({ headers: { Cookie: cookie } }))
      : axios.create({ headers: { Cookie: cookie } });
  }

  async deleteOwnOrg() {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/orgs/delete-own",
      validateStatus: (s) => s === 204,
    });
  }
}
