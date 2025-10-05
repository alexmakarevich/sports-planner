import axios, { AxiosInstance, AxiosPromise, AxiosRequestConfig } from "axios";
import { API_URL } from "./env";
import { makeTestAxios } from "../utils";
import z from "zod";

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

  async createUser({
    username,
    password,
  }: {
    username: string;
    password: string;
  }) {
    const { data } = await this.axios({
      method: "POST",
      url: API_URL + "/users/create",
      data: {
        username,
        password,
      },
    });
    return z.string().parse(data);
  }

  async deleteUserById(id: string) {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/users/delete-by-id/" + id,
    });
  }

  async deleteOwnOrg() {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/orgs/delete-own",
      validateStatus: (s) => s === 204,
    });
  }
}
