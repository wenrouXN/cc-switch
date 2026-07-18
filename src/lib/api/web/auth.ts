import { post } from "../web-client";

export interface WebLoginResult {
  token: string;
  mustChange: boolean;
}

export const authApi = {
  async login(password: string): Promise<WebLoginResult> {
    return post("/auth/login", { password });
  },
  async changePassword(
    currentPassword: string,
    newPassword: string,
  ): Promise<{ token: string }> {
    return post("/auth/change-password", { currentPassword, newPassword });
  },
  async logout(): Promise<void> {
    return post("/auth/logout", {});
  },
};
