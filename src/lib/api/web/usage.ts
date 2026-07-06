import { get, post } from "../web-client";
import type {
  UsageSummary,
  UsageSummaryByApp,
  DailyStats,
  ProviderStats,
  ModelStats,
  RequestLog,
  LogFilters,
  ModelPricing,
  ProviderLimitStatus,
  PaginatedLogs,
  SessionSyncResult,
  DataSourceSummary,
} from "@/types/usage";
import type { UsageResult } from "@/types";
import type { AppId } from "../types";
import type { TemplateType } from "@/config/constants";

export const usageApi = {
  query: async (providerId: string, appId: AppId): Promise<UsageResult> => {
    return get(`/usage/query?providerId=${encodeURIComponent(providerId)}&app=${encodeURIComponent(appId)}`);
  },

  testScript: async (
    providerId: string,
    appId: AppId,
    scriptCode: string,
    timeout?: number,
    apiKey?: string,
    baseUrl?: string,
    accessToken?: string,
    userId?: string,
    templateType?: TemplateType,
  ): Promise<UsageResult> => {
    return post("/usage/test-script", {
      providerId,
      app: appId,
      scriptCode,
      timeout,
      apiKey,
      baseUrl,
      accessToken,
      userId,
      templateType,
    });
  },

  getUsageSummary: async (
    startDate?: number,
    endDate?: number,
    appType?: string,
    providerName?: string,
    model?: string,
  ): Promise<UsageSummary> => {
    const params = new URLSearchParams();
    if (startDate) params.set("startDate", String(startDate));
    if (endDate) params.set("endDate", String(endDate));
    if (appType) params.set("appType", appType);
    if (providerName) params.set("providerName", providerName);
    if (model) params.set("model", model);
    return get(`/usage/summary?${params.toString()}`);
  },

  getUsageSummaryByApp: async (
    startDate?: number,
    endDate?: number,
    providerName?: string,
    model?: string,
  ): Promise<UsageSummaryByApp[]> => {
    const params = new URLSearchParams();
    if (startDate) params.set("startDate", String(startDate));
    if (endDate) params.set("endDate", String(endDate));
    if (providerName) params.set("providerName", providerName);
    if (model) params.set("model", model);
    return get(`/usage/summary-by-app?${params.toString()}`);
  },

  getUsageTrends: async (
    startDate?: number,
    endDate?: number,
    appType?: string,
    providerName?: string,
    model?: string,
  ): Promise<DailyStats[]> => {
    const params = new URLSearchParams();
    if (startDate) params.set("startDate", String(startDate));
    if (endDate) params.set("endDate", String(endDate));
    if (appType) params.set("appType", appType);
    if (providerName) params.set("providerName", providerName);
    if (model) params.set("model", model);
    return get(`/usage/trends?${params.toString()}`);
  },

  getProviderStats: async (
    startDate?: number,
    endDate?: number,
    appType?: string,
    providerName?: string,
    model?: string,
  ): Promise<ProviderStats[]> => {
    const params = new URLSearchParams();
    if (startDate) params.set("startDate", String(startDate));
    if (endDate) params.set("endDate", String(endDate));
    if (appType) params.set("appType", appType);
    if (providerName) params.set("providerName", providerName);
    if (model) params.set("model", model);
    return get(`/usage/provider-stats?${params.toString()}`);
  },

  getModelStats: async (
    startDate?: number,
    endDate?: number,
    appType?: string,
    providerName?: string,
    model?: string,
  ): Promise<ModelStats[]> => {
    const params = new URLSearchParams();
    if (startDate) params.set("startDate", String(startDate));
    if (endDate) params.set("endDate", String(endDate));
    if (appType) params.set("appType", appType);
    if (providerName) params.set("providerName", providerName);
    if (model) params.set("model", model);
    return get(`/usage/model-stats?${params.toString()}`);
  },

  getRequestLogs: async (
    filters: LogFilters,
    page: number = 0,
    pageSize: number = 20,
  ): Promise<PaginatedLogs> => {
    const params = new URLSearchParams();
    if (filters.startDate) params.set("startDate", String(filters.startDate));
    if (filters.endDate) params.set("endDate", String(filters.endDate));
    if (filters.appType) params.set("appType", filters.appType);
    if (filters.providerName) params.set("providerName", filters.providerName);
    if (filters.model) params.set("model", filters.model);
    if (filters.statusCode) params.set("statusCode", String(filters.statusCode));
    params.set("page", String(page));
    params.set("pageSize", String(pageSize));
    return get(`/usage/request-logs?${params.toString()}`);
  },

  getRequestDetail: async (requestId: string): Promise<RequestLog | null> => {
    return get(`/usage/request-detail?requestId=${encodeURIComponent(requestId)}`);
  },

  getModelPricing: async (): Promise<ModelPricing[]> => {
    return get("/usage/model-pricing");
  },

  updateModelPricing: async (
    modelId: string,
    displayName: string,
    inputCost: string,
    outputCost: string,
    cacheReadCost: string,
    cacheCreationCost: string,
  ): Promise<void> => {
    return post("/usage/model-pricing", {
      modelId,
      displayName,
      inputCost,
      outputCost,
      cacheReadCost,
      cacheCreationCost,
    });
  },

  deleteModelPricing: async (modelId: string): Promise<void> => {
    return post("/usage/model-pricing/delete", { modelId });
  },

  checkProviderLimits: async (
    providerId: string,
    appType: string,
  ): Promise<ProviderLimitStatus> => {
    return get(`/usage/provider-limits?providerId=${encodeURIComponent(providerId)}&appType=${encodeURIComponent(appType)}`);
  },

  syncSessionUsage: async (): Promise<SessionSyncResult> => {
    return post("/usage/sync", {});
  },

  getDataSourceBreakdown: async (): Promise<DataSourceSummary[]> => {
    return get("/usage/data-sources");
  },
};
