import { get, post } from "../web-client";
import type {
  ProviderHealth,
  CircuitBreakerConfig,
  CircuitBreakerStats,
  FailoverQueueItem,
} from "@/types/proxy";

export interface Provider {
  id: string;
  name: string;
  settingsConfig: unknown;
  websiteUrl?: string;
  category?: string;
  createdAt?: number;
  sortIndex?: number;
  notes?: string;
  meta?: unknown;
  icon?: string;
  iconColor?: string;
}

export const failoverApi = {
  // ========== 熔断器 API ==========

  async getProviderHealth(
    providerId: string,
    appType: string,
  ): Promise<ProviderHealth> {
    return get(`/failover/health/${encodeURIComponent(providerId)}?app=${encodeURIComponent(appType)}`);
  },

  async resetCircuitBreaker(
    providerId: string,
    appType: string,
  ): Promise<void> {
    await post("/failover/health", { appType, providerId });
  },

  async getCircuitBreakerConfig(): Promise<CircuitBreakerConfig> {
    return get("/failover/circuit-breaker/config");
  },

  async updateCircuitBreakerConfig(
    config: CircuitBreakerConfig,
  ): Promise<void> {
    await post("/failover/circuit-breaker/config", config);
  },

  async getCircuitBreakerStats(
    providerId: string,
    appType: string,
  ): Promise<CircuitBreakerStats | null> {
    return get(`/failover/circuit-breaker/stats?providerId=${encodeURIComponent(providerId)}&app=${encodeURIComponent(appType)}`);
  },

  // ========== 故障转移队列 API ==========

  async getFailoverQueue(appType: string): Promise<FailoverQueueItem[]> {
    return get(`/failover/queue?app=${encodeURIComponent(appType)}`);
  },

  async getAvailableProvidersForFailover(appType: string): Promise<Provider[]> {
    return get(`/failover/available?app=${encodeURIComponent(appType)}`);
  },

  async addToFailoverQueue(appType: string, providerId: string): Promise<void> {
    await post("/failover/queue", { appType, providerId });
  },

  async removeFromFailoverQueue(
    appType: string,
    providerId: string,
  ): Promise<void> {
    await post("/failover/queue/remove", { appType, providerId });
  },

  async getAutoFailoverEnabled(appType: string): Promise<boolean> {
    return get(`/failover/enabled?app=${encodeURIComponent(appType)}`);
  },

  async setAutoFailoverEnabled(
    appType: string,
    enabled: boolean,
  ): Promise<void> {
    await post("/failover/enabled", { appType, enabled });
  },
};
