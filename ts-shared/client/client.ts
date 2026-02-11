import axios, { AxiosInstance, AxiosPromise, AxiosRequestConfig } from "axios";

import z from "zod";

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

export type ClientKind = "node" | "browser";

export class Client {
  cookie: string;
  axios: AxiosInstance;
  API_URL: string;
  kind: ClientKind;
  allCookies: string[];
  constructor({
    cookie,
    API_URL,
    kind,
  }: {
    cookie: string;
    API_URL: string;
    kind: ClientKind;
  }) {
    this.cookie = cookie;
    this.API_URL = API_URL;
    this.kind = kind;
    this.allCookies = [this.cookie];

    if (kind === "node") {
      this.axios = axios.create({
        baseURL: API_URL,
        withCredentials: true,
      });

      this.axios.interceptors.request.use((config) => {
        // naively setting all cookies
        // TODO: stricter
        if (this.allCookies.length > 0) {
          config.headers.Cookie = this.allCookies.join("; ");
        }
        return config;
      });

      this.axios.interceptors.response.use((response) => {
        // naively setting all cookies
        // TODO: stricter

        const setCookie = response.headers["set-cookie"];
        if (setCookie) {
          this.allCookies = setCookie;
        }
        return response;
      });
    } else {
      // browser
      this.axios = axios.create({ baseURL: API_URL, withCredentials: true });
    }
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
      url: "/users/create",
      data: {
        username,
        password,
      },
    });
    return z.string().parse(data);
  }

  async listUsers() {
    const { status, data } = await this.axios({
      url: "/users/list",
    });
    return listUsersResponseSchema.parse(data);
  }

  async deleteUserById(id: string) {
    await this.axios({
      method: "DELETE",
      url: "/users/delete-by-id/" + id,
    });
  }

  async deleteOwnUser() {
    await this.axios({
      method: "DELETE",
      url: "/users/delete-own",
    });
  }

  // ROLES

  async listRoles() {
    const { data } = await this.axios({
      method: "GET",
      url: "/roles/list",
    });
    return listRolesResSchema.parse(data);
  }

  async listOwnRoles() {
    const { data } = await this.axios({
      method: "GET",
      url: "/roles/list-own",
    });
    return z.array(roleSchema).parse(data);
  }

  async assignRole({ user_id, role }: { user_id: string; role: Role }) {
    await this.axios({
      method: "POST",
      url: "/roles/assign",
      data: { user_id, role },
    });
  }

  async unassignRole({ user_id, role }: { user_id: string; role: Role }) {
    await this.axios({
      method: "DELETE",
      url: "/roles/unassign",
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
      url: "/invites-to-org/create",
    });
    return z.string().parse(data);
  }

  async deleteServiceInviteById(id: string) {
    await this.axios({
      method: "DELETE",
      url: "/invites-to-org/delete-by-id/" + id,
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
      url: `${this.API_URL}/teams/list`,
    });
    return this.listTeamsResponseSchema.parse(data);
  }

  async getTeam(id: string): Promise<Team> {
    const { data } = await this.axios({
      method: "get",
      url: `${this.API_URL}/teams/get/${id}`,
    });
    return this.teamSchema.parse(data);
  }

  async createTeam(payload: { name: string; slug: string }): Promise<string> {
    const { data } = await this.axios({
      method: "post",
      url: `${this.API_URL}/teams/create`,
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
      url: `${this.API_URL}/teams/update/${id}`,
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
      url: `${this.API_URL}/teams/delete-by-id/${id}`,
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
      url: "/games/create",
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
      url: "/games/delete-by-id/" + gameId,
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
      url: "/games/list-for-team/" + teamId,
    });

    return this.listGamesResponse.parse(data);
  }

  // EVENT INVITE

  async listOwnInvites() {
    const { data } = await this.axios({
      method: "GET",
      url: "/game-invites/list-own",
    });
    return listOwnInvitesResSchema.parse(data);
  }

  async listInvitesToGame(game_id: string) {
    const { data } = await this.axios({
      method: "GET",
      url: "/game-invites/list-to-game/" + game_id,
    });
    return listInvitesToGameResSchema.parse(data);
  }

  async respondToInvite(payload: {
    invite_id: string;
    response: InviteResponseFromUser;
  }) {
    await this.axios({
      method: "POST",
      url: "/game-invites/respond",
      data: payload,
    });
    return;
  }

  // LOG-OUT

  async logOut() {
    await this.axios({
      method: "POST",
      url: "/log-out",
    });
  }

  // SElF-DELETE

  async deleteOwnOrg() {
    await this.axios({
      method: "DELETE",
      url: "/orgs/delete-own",
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
