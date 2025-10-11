import axios, { AxiosInstance, AxiosPromise, AxiosRequestConfig } from "axios";
import { API_URL } from "./env";
import { makeTestAxios } from "../utils";
import z from "zod";
import path from "path";

z.config({
  customError: (issue) => {
    const { issues, path, input } = issue;
    const formattedPath = path?.join("/");
    return JSON.stringify({ formattedPath, issues, input }, null, 2);
  },
});

const listUsersResponseSchema = z.array(
  z.object({
    id: z.string(),
    username: z.string(),
  })
);

export type Role = "SuperAdmin" | "OrgAdmin" | "Coach" | "Player";
export const roleSchema = z.enum(["SuperAdmin", "OrgAdmin", "Coach", "Player"]);

const listRolesResSchema = z.record(z.string(), z.array(roleSchema));

export class Client {
  cookie: string;
  ownId: string;
  axios: (x: AxiosRequestConfig) => AxiosPromise;
  constructor({
    cookie,
    isTest,
    ownId,
  }: {
    cookie: string;
    isTest?: true;
    ownId: string;
  }) {
    console.log("new Client", { cookie, isTest });

    this.cookie = cookie;
    this.ownId = ownId;
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

  // ROLES

  async listRoles() {
    const { data } = await this.axios({
      method: "GET",
      url: API_URL + "/roles/list",
    });
    return listRolesResSchema.parse(data);
  }

  async listOwnRoles() {
    const { data } = await this.axios({
      method: "GET",
      url: API_URL + "/roles/list-own",
    });
    return z.array(roleSchema).parse(data);
  }

  async assignRole({ user_id, role }: { user_id: string; role: Role }) {
    await this.axios({
      method: "POST",
      url: API_URL + "/roles/assign",
      data: { user_id, role },
    });
  }

  async unassignRole({ user_id, role }: { user_id: string; role: Role }) {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/roles/unassign",
      data: { user_id, role },
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
      url: API_URL + "/invites-to-org/create",
    });
    return z.string().parse(data);
  }

  async deleteServiceInviteById(id: string) {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/invites-to-org/delete-by-id/" + id,
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
