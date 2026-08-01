import { del, get, post, put } from "../web-client";
import type {
  Profile,
  ProfileScope,
  ProfilesResponse,
} from "../profiles";

export const profilesApi = {
  async list(): Promise<ProfilesResponse> {
    return get("/profiles");
  },

  async create(name: string, scope: ProfileScope): Promise<Profile> {
    return post("/profiles", { name, scope });
  },

  async update(
    id: string,
    options: { name?: string; resnapshot?: boolean; scope?: ProfileScope },
  ): Promise<Profile> {
    return put(`/profiles/${encodeURIComponent(id)}`, options);
  },

  async delete(id: string): Promise<void> {
    return del(`/profiles/${encodeURIComponent(id)}`);
  },

  async apply(id: string, scope: ProfileScope): Promise<string[]> {
    return post(`/profiles/${encodeURIComponent(id)}/apply`, { scope });
  },

  async clearCurrent(scope: ProfileScope): Promise<void> {
    return post("/profiles/clear", { scope });
  },
};
