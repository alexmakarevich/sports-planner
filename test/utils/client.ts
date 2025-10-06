import axios, { AxiosInstance, AxiosPromise, AxiosRequestConfig } from "axios";
import { API_URL } from "./env";
import { makeTestAxios } from "../utils";
import z from "zod";

const listUsersResponseSchema = z.array(
  z.object({
    id: z.string(),
    username: z.string(),
  })
);

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

  // USER

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

  async listUsers() {
    const { status, data } = await this.axios({
      url: API_URL + "/users/list",
    });
    return listUsersResponseSchema.parse(data);
  }

  async deleteUserById(id: string) {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/users/delete-by-id/" + id,
    });
  }

  async deleteOwnUser() {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/users/delete-own",
    });
  }

  // SERVICE INVITES

  /**
   *
   * @returns {string} ID of invite
   */
  async createServiceInvite(): Promise<string> {
    const { data } = await this.axios({
      method: "POST",
      url: API_URL + "/service-invites/create",
    });
    return z.string().parse(data);
  }

  async deleteServiceInviteById(id: string) {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/service-invites/delete-by-id/" + id,
    });
  }

  // LOG-OUT

  async logOut() {
    await this.axios({
      method: "POST",
      url: API_URL + "/log-out",
    });
  }

  // SElF-DELETE

  async deleteOwnOrg() {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/orgs/delete-own",
      validateStatus: (s) => s === 204,
    });
  }
}
