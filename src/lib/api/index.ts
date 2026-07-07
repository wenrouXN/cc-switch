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
import { vscodeApi } from "./vscode";
import { authApi } from "./auth";
import { subscriptionApi } from "./subscription";

export type { AppId } from "./types";
export type { ProviderSwitchEvent } from "./providers";
export type { Prompt } from "./prompts";
export type { GitHubAccount } from "./copilot";
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
  _tauriLoaded = true;
}

const useTauri = isTauri();

// If in Tauri mode, load Tauri APIs immediately (fire-and-forget)
if (useTauri) {
  loadTauriApis();
}

// Helper to get Tauri API or fall back to web API
function pickApi<T>(tauriApi: T | null, webApi: T): T {
  return useTauri && tauriApi !== null ? tauriApi : webApi;
}

// Export APIs — always available (web APIs as fallback)
export const providersApi = pickApi(_tauriProvidersApi, webProvidersApi);
export const universalProvidersApi = pickApi(
  _tauriUniversalProvidersApi,
  webUniversalProvidersApi,
);
export const settingsApi = pickApi(_tauriSettingsApi, webSettingsApi);
export const backupsApi = pickApi(_tauriBackupsApi, webBackupsApi);
export const mcpApi = pickApi(_tauriMcpApi, webMcpApi);
export const configApi = pickApi(_tauriConfigApi, webConfigApi);
export const promptsApi = pickApi(_tauriPromptsApi, webPromptsApi);
export const skillsApi = pickApi(_tauriSkillsApi, webSkillsApi);
export const proxyApi = pickApi(_tauriProxyApi, webProxyApi);
export const failoverApi = pickApi(_tauriFailoverApi, webFailoverApi);
export const sessionsApi = pickApi(_tauriSessionsApi, webSessionsApi);
export const openclawApi = pickApi(_tauriOpenclawApi, webOpenclawApi);
export const hermesApi = pickApi(_tauriHermesApi, webHermesApi);
export const usageApi = pickApi(_tauriUsageApi, webUsageApi);
export { vscodeApi };
export const workspaceApi = pickApi(_tauriWorkspaceApi, webWorkspaceApi);
export { authApi };
export * as copilotApi from "./copilot";
export { subscriptionApi };
