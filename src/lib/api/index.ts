import { isTauri } from "@/lib/environment";

// Web APIs (always safe to import - no Tauri dependency)
import {
  providersApi as webProvidersApi,
  universalProvidersApi as webUniversalProvidersApi,
} from "./web/providers";
import { settingsApi as webSettingsApi } from "./web/settings";
import { backupsApi as webBackupsApi } from "./web/settings";
import { mcpApi as webMcpApi } from "./web/mcp";
import * as webConfigApi from "./web/config";
import { promptsApi as webPromptsApi } from "./web/prompts";
import { skillsApi as webSkillsApi } from "./web/skills";
import { proxyApi as webProxyApi } from "./web/proxy";
import { failoverApi as webFailoverApi } from "./web/failover";
import { sessionsApi as webSessionsApi } from "./web/sessions";
import { openclawApi as webOpenclawApi } from "./web/openclaw";
import { hermesApi as webHermesApi } from "./web/hermes";
import { usageApi as webUsageApi } from "./web/usage";
import { workspaceApi as webWorkspaceApi } from "./web/workspace";
import { profilesApi as webProfilesApi } from "./web/profiles";
import { authApi as webAuthApi } from "./web/auth";
import { subscriptionApi as webSubscriptionApi } from "./web/subscription";
import { vscodeApi } from "./vscode";

export type { AppId } from "./types";
export type { ProviderSwitchEvent } from "./providers";
export type { Prompt } from "./prompts";
export type { Profile, ProfilePayload, ProfilesResponse } from "./profiles";
export type { DailyMemoryFileInfo, DailyMemorySearchResult } from "./workspace";
export type {
  CopilotDeviceCodeResponse,
  CopilotAuthStatus,
  GitHubAccount,
} from "./copilot";
export type {
  ManagedAuthProvider,
  ManagedAuthStatus,
  ManagedAuthDeviceCodeResponse,
} from "./auth";

// Lazy Tauri API loading — only import Tauri modules when actually running in Tauri.
// This avoids `invoke is undefined` errors in web mode because the Tauri modules
// import `@tauri-apps/api/core` which doesn't work in a browser context.
let _tauriLoaded = false;
let _tauriProvidersApi: any = null;
let _tauriUniversalProvidersApi: any = null;
let _tauriSettingsApi: any = null;
let _tauriBackupsApi: any = null;
let _tauriMcpApi: any = null;
let _tauriConfigApi: any = null;
let _tauriPromptsApi: any = null;
let _tauriSkillsApi: any = null;
let _tauriProxyApi: any = null;
let _tauriFailoverApi: any = null;
let _tauriSessionsApi: any = null;
let _tauriOpenclawApi: any = null;
let _tauriHermesApi: any = null;
let _tauriUsageApi: any = null;
let _tauriWorkspaceApi: any = null;
let _tauriProfilesApi: any = null;
let _tauriAuthApi: any = null;
let _tauriSubscriptionApi: any = null;

async function loadTauriApis() {
  if (_tauriLoaded) return;
  const [
    providers,
    settings,
    mcp,
    prompts,
    skills,
    proxy,
    failover,
    sessions,
    openclaw,
    hermes,
    usage,
    workspace,
    profiles,
    auth,
    subscription,
    config,
  ] = await Promise.all([
    import("./providers"),
    import("./settings"),
    import("./mcp"),
    import("./prompts"),
    import("./skills"),
    import("./proxy"),
    import("./failover"),
    import("./sessions"),
    import("./openclaw"),
    import("./hermes"),
    import("./usage"),
    import("./workspace"),
    import("./profiles"),
    import("./auth"),
    import("./subscription"),
    import("./config"),
  ]);
  _tauriProvidersApi = providers.providersApi;
  _tauriUniversalProvidersApi = providers.universalProvidersApi;
  _tauriSettingsApi = settings.settingsApi;
  _tauriBackupsApi = settings.backupsApi;
  _tauriMcpApi = mcp.mcpApi;
  _tauriConfigApi = config;
  _tauriPromptsApi = prompts.promptsApi;
  _tauriSkillsApi = skills.skillsApi;
  _tauriProxyApi = proxy.proxyApi;
  _tauriFailoverApi = failover.failoverApi;
  _tauriSessionsApi = sessions.sessionsApi;
  _tauriOpenclawApi = openclaw.openclawApi;
  _tauriHermesApi = hermes.hermesApi;
  _tauriUsageApi = usage.usageApi;
  _tauriWorkspaceApi = workspace.workspaceApi;
  _tauriProfilesApi = profiles.profilesApi;
  _tauriAuthApi = auth.authApi;
  _tauriSubscriptionApi = subscription.subscriptionApi;
  _tauriLoaded = true;
}

const useTauri = isTauri();

// If in Tauri mode, load Tauri APIs immediately (fire-and-forget)
if (useTauri) {
  loadTauriApis();
}

// Tauri modules load asynchronously. Proxy each method so desktop calls wait for
// that load, while web calls go straight to the REST implementation.
function pickApi<T extends object>(getTauriApi: () => T | null, webApi: T): T {
  if (!useTauri) return webApi;

  return new Proxy(webApi, {
    get(_target, property) {
      return async (...args: unknown[]) => {
        await loadTauriApis();
        const tauriApi = getTauriApi();
        const method = tauriApi?.[property as keyof T];
        if (typeof method !== "function") {
          throw new Error(`Tauri API method is unavailable: ${String(property)}`);
        }
        return method.apply(tauriApi, args);
      };
    },
  });
}

// Export APIs — always available (web APIs as fallback)
export const providersApi = pickApi(
  () => _tauriProvidersApi,
  webProvidersApi,
);
export const universalProvidersApi = pickApi(
  () => _tauriUniversalProvidersApi,
  webUniversalProvidersApi,
);
export const settingsApi = pickApi(() => _tauriSettingsApi, webSettingsApi);
export const backupsApi = pickApi(() => _tauriBackupsApi, webBackupsApi);
export const mcpApi = pickApi(() => _tauriMcpApi, webMcpApi);
export const configApi = pickApi(() => _tauriConfigApi, webConfigApi);
export const promptsApi = pickApi(() => _tauriPromptsApi, webPromptsApi);
export const skillsApi = pickApi(() => _tauriSkillsApi, webSkillsApi);
export const proxyApi = pickApi(() => _tauriProxyApi, webProxyApi);
export const failoverApi = pickApi(() => _tauriFailoverApi, webFailoverApi);
export const sessionsApi = pickApi(() => _tauriSessionsApi, webSessionsApi);
export const openclawApi = pickApi(() => _tauriOpenclawApi, webOpenclawApi);
export const hermesApi = pickApi(() => _tauriHermesApi, webHermesApi);
export const usageApi = pickApi(() => _tauriUsageApi, webUsageApi);
export { vscodeApi };
export const workspaceApi = pickApi(
  () => _tauriWorkspaceApi,
  webWorkspaceApi,
);
export const profilesApi = pickApi(() => _tauriProfilesApi, webProfilesApi);
export const authApi = pickApi(() => _tauriAuthApi, webAuthApi);
export * as copilotApi from "./copilot";
export const subscriptionApi = pickApi(
  () => _tauriSubscriptionApi,
  webSubscriptionApi,
);
