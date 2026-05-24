import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { deepseekProxyApi } from "@/lib/api";
import { extractErrorMessage } from "@/utils/errorUtils";

export function useDeepSeekProxy() {
  const queryClient = useQueryClient();
  const { t } = useTranslation();

  const { data: status, isLoading } = useQuery({
    queryKey: ["deepseekProxyStatus"],
    queryFn: () => deepseekProxyApi.getStatus(),
    refetchInterval: (query) => (query.state.data?.running ? 5000 : false),
    placeholderData: (previousData) => previousData,
  });

  const startMutation = useMutation({
    mutationFn: () => deepseekProxyApi.start(),
    onSuccess: (result) => {
      toast.success(
        t("deepseekProxy.started", {
          port: result.port,
          defaultValue: `DeepSeek 代理已启动 (端口 ${result.port})`,
        }),
        { closeButton: true },
      );
      queryClient.invalidateQueries({ queryKey: ["deepseekProxyStatus"] });
    },
    onError: (error: Error) => {
      const detail = extractErrorMessage(error) || "未知错误";
      toast.error(
        t("deepseekProxy.startFailed", {
          detail,
          defaultValue: `启动 DeepSeek 代理失败: ${detail}`,
        }),
      );
    },
  });

  const stopMutation = useMutation({
    mutationFn: () => deepseekProxyApi.stop(),
    onSuccess: () => {
      toast.success(
        t("deepseekProxy.stopped", { defaultValue: "DeepSeek 代理已停止" }),
        { closeButton: true },
      );
      queryClient.invalidateQueries({ queryKey: ["deepseekProxyStatus"] });
    },
    onError: (error: Error) => {
      const detail = extractErrorMessage(error) || "未知错误";
      toast.error(
        t("deepseekProxy.stopFailed", {
          detail,
          defaultValue: `停止 DeepSeek 代理失败: ${detail}`,
        }),
      );
    },
  });

  return {
    status,
    isLoading,
    isRunning: status?.running ?? false,
    start: startMutation.mutateAsync,
    stop: stopMutation.mutateAsync,
    isPending: startMutation.isPending || stopMutation.isPending,
  };
}
