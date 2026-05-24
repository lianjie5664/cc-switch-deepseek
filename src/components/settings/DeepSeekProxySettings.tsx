import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Loader2,
  Save,
  Eye,
  EyeOff,
  Activity,
  Power,
} from "lucide-react";
import { useDeepSeekProxy } from "@/hooks/useDeepSeekProxy";
import { useSettingsQuery, useSaveSettingsMutation } from "@/lib/query";
import type { Settings, DeepSeekProxyConfig } from "@/types";

export function DeepSeekProxySettings() {
  const { t } = useTranslation();
  const { data: settingsData } = useSettingsQuery();
  const saveMutation = useSaveSettingsMutation();
  const { isRunning, start, stop, isPending: isProxyPending } = useDeepSeekProxy();

  const saved = settingsData?.deepseekProxy;

  const [apiKey, setApiKey] = useState("");
  const [port, setPort] = useState(11435);
  const [model, setModel] = useState("deepseek-v4-pro");
  const [enabled, setEnabled] = useState(false);
  const [showKey, setShowKey] = useState(false);
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    if (saved) {
      setApiKey(saved.apiKey ?? "");
      setPort(saved.port ?? 11435);
      setModel(saved.model ?? "deepseek-v4-pro");
      setEnabled(saved.enabled ?? false);
      setDirty(false);
    }
  }, [saved]);

  const markDirty = useCallback(() => setDirty(true), []);

  const handleSave = useCallback(async () => {
    const dp: DeepSeekProxyConfig = {
      enabled,
      port: port || 11435,
      apiKey,
      model: model || "deepseek-v4-pro",
    };

    const merged: Settings = {
      ...(settingsData ?? {}),
      deepseekProxy: dp,
    } as Settings;

    try {
      await saveMutation.mutateAsync(merged as any);
      setDirty(false);

      if (enabled) {
        await start();
      } else {
        if (isRunning) {
          await stop();
        }
      }
    } catch (error) {
      console.error("Save DeepSeek proxy config failed:", error);
    }
  }, [enabled, port, apiKey, model, settingsData, saveMutation, start, stop, isRunning]);

  const handleToggleEnabled = useCallback(
    (checked: boolean) => {
      setEnabled(checked);
      setDirty(true);
    },
    [],
  );

  return (
    <div className="space-y-4">
      {/* Enable toggle */}
      <div className="flex items-center justify-between rounded-xl border border-border bg-card/50 p-4 transition-colors hover:bg-muted/50">
        <div className="flex items-center gap-3">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-background ring-1 ring-border">
            <Power className="h-4 w-4 text-blue-500" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium leading-none">
              {t("deepseekProxy.enable", { defaultValue: "DeepSeek 本地代理" })}
            </p>
            <p className="text-xs text-muted-foreground">
              {isRunning
                ? t("deepseekProxy.statusRunning", {
                    port,
                    defaultValue: `运行中 — 端口 ${port}`,
                  })
                : t("deepseekProxy.statusStopped", {
                    defaultValue: "未启动",
                  })}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {isRunning && (
            <Badge variant="default" className="gap-1 h-6">
              <Activity className="h-3 w-3 animate-pulse" />
              {t("deepseekProxy.running", { defaultValue: "运行中" })}
            </Badge>
          )}
          <Switch
            checked={enabled}
            onCheckedChange={handleToggleEnabled}
            disabled={isProxyPending}
          />
        </div>
      </div>

      {/* Config fields */}
      <div className="rounded-lg border border-border bg-muted/40 p-4 space-y-4">
        {/* API Key */}
        <div className="space-y-2">
          <Label htmlFor="deepseek-api-key">
            {t("deepseekProxy.apiKey", { defaultValue: "API Key" })}
          </Label>
          <div className="relative">
            <Input
              id="deepseek-api-key"
              type={showKey ? "text" : "password"}
              value={apiKey}
              onChange={(e) => {
                setApiKey(e.target.value);
                markDirty();
              }}
              placeholder="sk-..."
              className="font-mono text-sm pr-10"
            />
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
              onClick={() => setShowKey(!showKey)}
              tabIndex={-1}
            >
              {showKey ? (
                <EyeOff className="h-4 w-4 text-muted-foreground" />
              ) : (
                <Eye className="h-4 w-4 text-muted-foreground" />
              )}
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            {t("deepseekProxy.apiKeyHint", {
              defaultValue: "DeepSeek API Key，从 platform.deepseek.com 获取",
            })}
          </p>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          {/* Port */}
          <div className="space-y-2">
            <Label htmlFor="deepseek-port">
              {t("deepseekProxy.port", { defaultValue: "本地端口" })}
            </Label>
            <Input
              id="deepseek-port"
              type="number"
              value={port}
              onChange={(e) => {
                setPort(parseInt(e.target.value) || 11435);
                markDirty();
              }}
              placeholder="11435"
            />
            <p className="text-xs text-muted-foreground">
              {t("deepseekProxy.portHint", {
                defaultValue: "代理服务监听的本地端口",
              })}
            </p>
          </div>

          {/* Model */}
          <div className="space-y-2">
            <Label htmlFor="deepseek-model">
              {t("deepseekProxy.model", { defaultValue: "模型" })}
            </Label>
            <Input
              id="deepseek-model"
              value={model}
              onChange={(e) => {
                setModel(e.target.value);
                markDirty();
              }}
              placeholder="deepseek-v4-pro"
            />
            <p className="text-xs text-muted-foreground">
              {t("deepseekProxy.modelHint", {
                defaultValue: "使用 deepseek-v4-pro 或 deepseek-v4-flash",
              })}
            </p>
          </div>
        </div>

        {/* Save button */}
        <div className="flex justify-end">
          <Button
            size="sm"
            onClick={handleSave}
            disabled={!dirty || saveMutation.isPending || isProxyPending}
          >
            {(saveMutation.isPending || isProxyPending) ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                {t("common.saving", { defaultValue: "保存中..." })}
              </>
            ) : (
              <>
                <Save className="mr-2 h-4 w-4" />
                {t("common.save", { defaultValue: "保存" })}
              </>
            )}
          </Button>
        </div>
      </div>

      {/* Info hint */}
      <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
        <p className="text-xs text-blue-600 dark:text-blue-400">
          {t("deepseekProxy.infoHint", {
            defaultValue:
              "启用后，本地将启动一个 DeepSeek 协议转换代理服务（需安装 Node.js）。Codex CLI 可通过此代理接入 DeepSeek 模型。",
          })}
        </p>
      </div>
    </div>
  );
}
