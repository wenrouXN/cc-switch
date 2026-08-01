import { get, post } from "../web-client";
import type { UsageResult } from "@/types";
import type { SubscriptionQuota } from "@/types/subscription";

export const subscriptionApi = {
  getQuota: (tool: string): Promise<SubscriptionQuota> =>
    get(`/subscription/quota?tool=${encodeURIComponent(tool)}`),

  getCodexOauthQuota: (_accountId: string | null): Promise<SubscriptionQuota> =>
    get("/subscription/oauth-quota?tool=codex_oauth"),

  getXaiOauthQuota: (_accountId: string | null): Promise<SubscriptionQuota> =>
    get("/subscription/oauth-quota?tool=xai_oauth"),

  getCodingPlanQuota: (
    baseUrl: string,
    apiKey: string,
    accessKeyId?: string,
    secretAccessKey?: string,
    codingPlanProvider?: string,
    teamOrganizationId?: string,
    teamProjectId?: string,
  ): Promise<SubscriptionQuota> =>
    post("/subscription/coding-plan", {
      baseUrl,
      apiKey,
      accessKeyId,
      secretAccessKey,
      codingPlanProvider,
      teamOrganizationId,
      teamProjectId,
    }),

  getBalance: (baseUrl: string, apiKey: string): Promise<UsageResult> =>
    post("/subscription/balance", { baseUrl, apiKey }),
};
