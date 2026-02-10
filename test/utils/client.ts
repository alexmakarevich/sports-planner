import axios, { AxiosPromise, AxiosRequestConfig } from "axios";
import { API_URL } from "./env";
import { makeTestAxios } from "../utils";
import z from "zod";
import { log } from "console";

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
  }),
);

export type Role = "super_admin" | "org_admin" | "coach" | "player";
export const roleSchema = z.enum([
  "super_admin",
  "org_admin",
  "coach",
  "player",
]);

export type LocationKind = "home" | "away" | "other";

const listRolesResSchema = z.record(z.string(), z.array(roleSchema));

export type Team = {
  id: string;
  org_id: string;
  name: string;
  slug: string;
};

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

  // TEAM

  private teamSchema = z.object({
    id: z.string(),
    org_id: z.string(),
    name: z.string(),
    slug: z.string(),
  });

  private listTeamsResponseSchema = z.array(this.teamSchema);

  async listTeams(): Promise<Team[]> {
    const { data } = await this.axios({
      method: "get",
      url: `${API_URL}/teams/list`,
    });
    return this.listTeamsResponseSchema.parse(data);
  }

  async getTeam(id: string): Promise<Team> {
    const { data } = await this.axios({
      method: "get",
      url: `${API_URL}/teams/get/${id}`,
    });
    return this.teamSchema.parse(data);
  }

  async createTeam(payload: { name: string; slug: string }): Promise<string> {
    const { data } = await this.axios({
      method: "post",
      url: `${API_URL}/teams/create`,
      data: payload,
    });
    return z.string().parse(data);
  }

  async updateTeam(
    id: string,
    payload: { name?: string; slug?: string },
  ): Promise<Team> {
    const { data } = await this.axios({
      method: "put",
      url: `${API_URL}/teams/update/${id}`,
      data: payload,
    });
    return this.teamSchema.parse(data);
  }

  // --------------------------------------------------------------------
  // Delete a team by id
  // --------------------------------------------------------------------
  async deleteTeamById(id: string): Promise<void> {
    await this.axios({
      method: "delete",
      url: `${API_URL}/teams/delete-by-id/${id}`,
    });
  }

  // GAME

  async createGame({
    team_id,
    opponent,
    start_time,
    stop_time,
    location,
    location_kind,
    invited_roles,
  }: {
    team_id: string;
    opponent: string;
    start_time: Date;
    stop_time?: Date;
    location: string;
    location_kind: LocationKind;
    invited_roles: Role[];
  }) {
    const { data } = await this.axios({
      method: "POST",
      url: API_URL + "/games/create",
      data: {
        team_id,
        opponent,
        start_time,
        stop_time,
        location,
        location_kind,
        invited_roles,
      },
    });
    return z.string().parse(data);
  }

  async deleteGame(gameId: string) {
    await this.axios({
      method: "DELETE",
      url: API_URL + "/games/delete-by-id/" + gameId,
    });
  }

  private listGamesResponse = z.array(
    z.object({
      id: z.string(),
      team_id: z.string(),
      opponent: z.string(),
      start_time: z.coerce.date(),
      stop_time: z.coerce.date().nullable(),
      location: z.string(),
      location_kind: z.enum(["home", "away", "other"]),
    }),
  );

  // Add the listGamesForTeam method
  async listGamesForTeam(teamId: string) {
    const { data } = await this.axios({
      method: "GET",
      url: API_URL + "/games/list-for-team/" + teamId,
    });

    log({ raw: data });
    return this.listGamesResponse.parse(data);
  }

  // EVENT INVITE

  async listOwnInvites() {
    const { data } = await this.axios({
      method: "GET",
      url: API_URL + "/game-invites/list-own",
    });
    return listOwnInvitesResSchema.parse(data);
  }

  async listInvitesToGame(game_id: string) {
    const { data } = await this.axios({
      method: "GET",
      url: API_URL + "/game-invites/list-to-game/" + game_id,
    });
    return listInvitesToGameResSchema.parse(data);
  }

  async respondToInvite(payload: {
    invite_id: string;
    response: InviteResponseFromUser;
  }) {
    await this.axios({
      method: "POST",
      url: API_URL + "/game-invites/respond",
      data: payload,
    });
    return;
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

export type InviteResponse = "pending" | "accepted" | "declined" | "unsure";
export type InviteResponseFromUser = "accepted" | "declined" | "unsure";

const listOwnInvitesResSchema = z.array(
  z.object({
    invite_id: z.string(),
    game_id: z.string(),
    opponent: z.string(),
    response: z.union([
      z.literal("pending"),
      z.literal("accepted"),
      z.literal("declined"),
      z.literal("unsure"),
    ]),
  }),
);

const listInvitesToGameResSchema = z.array(
  z.object({
    invite_id: z.string(),
    user_id: z.string(),
    username: z.string(),
    response: z.union([
      z.literal("pending"),
      z.literal("accepted"),
      z.literal("declined"),
      z.literal("unsure"),
    ]),
  }),
);
