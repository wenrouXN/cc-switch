import { post } from "../web-client";
import type {
  ManagedAuthAccount,
  ManagedAuthDeviceCodeResponse,
  ManagedAuthProvider,
  ManagedAuthStatus,
} from "../auth";

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
  async authGetStatus(
    authProvider: ManagedAuthProvider,
  ): Promise<ManagedAuthStatus> {
    return {
      provider: authProvider,
      authenticated: false,
      default_account_id: null,
      migration_error: null,
      accounts: [],
    };
  },
  async authListAccounts(
    _authProvider: ManagedAuthProvider,
  ): Promise<ManagedAuthAccount[]> {
    return [];
  },
  async authStartLogin(
    _authProvider: ManagedAuthProvider,
    _githubDomain?: string,
  ): Promise<ManagedAuthDeviceCodeResponse> {
    throw new Error("Managed OAuth is only available in the desktop app");
  },
  async authPollForAccount(
    _authProvider: ManagedAuthProvider,
    _deviceCode: string,
    _githubDomain?: string,
  ): Promise<ManagedAuthAccount | null> {
    return null;
  },
  async authRemoveAccount(
    _authProvider: ManagedAuthProvider,
    _accountId: string,
  ): Promise<void> {},
  async authSetDefaultAccount(
    _authProvider: ManagedAuthProvider,
    _accountId: string,
  ): Promise<void> {},
  async authLogout(_authProvider: ManagedAuthProvider): Promise<void> {},
};
