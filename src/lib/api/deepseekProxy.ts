import { invoke } from "@tauri-apps/api/core";
import type { DeepSeekProxyStatus } from "@/types";

export const deepseekProxyApi = {
  /** 获取 DeepSeek 代理状态 */
  async getStatus(): Promise<DeepSeekProxyStatus> {
    return invoke("get_deepseek_proxy_status");
  },

  /** 启动 DeepSeek 代理 */
  async start(): Promise<DeepSeekProxyStatus> {
    return invoke("start_deepseek_proxy");
  },

  /** 停止 DeepSeek 代理 */
  async stop(): Promise<DeepSeekProxyStatus> {
    return invoke("stop_deepseek_proxy");
  },
};
