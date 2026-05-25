import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useState } from "react";
import { Save, Key, Cpu, Globe } from "lucide-react";

interface AppSetting {
  key: string;
  value: string;
  updated_at: string;
}

async function api<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    throw new Error(`Failed to call ${cmd}`);
  }
}

const AI_PROVIDERS = [
  { value: "openai", label: "OpenAI" },
  { value: "anthropic", label: "Anthropic (Claude)" },
  { value: "google", label: "Google (Gemini)" },
  { value: "deepseek", label: "DeepSeek" },
  { value: "custom", label: "Custom (OpenAI-compatible)" },
];

export function SettingsPage() {
  const queryClient = useQueryClient();

  const { data: settings, isLoading } = useQuery({
    queryKey: ["settings"],
    queryFn: () => api<AppSetting[]>("get_all_settings"),
  });

  // Form state
  const [provider, setProvider] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState("");

  // Initialize form when settings load
  const settingsMap = Object.fromEntries(
    (settings ?? []).map((s) => [s.key, s.value]),
  );

  // Sync form on first load
  const [initialized, setInitialized] = useState(false);
  if (!initialized && settings && settings.length > 0) {
    setProvider(settingsMap.ai_provider ?? "openai");
    setApiKey(settingsMap.ai_api_key ?? "");
    setModel(settingsMap.ai_model ?? "gpt-4o");
    setBaseUrl(settingsMap.ai_base_url ?? "");
    setInitialized(true);
  }

  const saveMutation = useMutation({
    mutationFn: async () => {
      setError("");
      await api("set_setting", { key: "ai_provider", value: provider });
      await api("set_setting", { key: "ai_api_key", value: apiKey });
      await api("set_setting", { key: "ai_model", value: model });
      await api("set_setting", { key: "ai_base_url", value: baseUrl });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    },
    onError: (err) => {
      setError(err instanceof Error ? err.message : "Failed to save");
    },
  });

  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <h1 className="text-2xl font-bold">Settings</h1>
        <p className="text-muted-foreground">
          Configure your AI provider for automatic analysis and report generation
        </p>
      </div>

      {isLoading ? (
        <div className="space-y-4">
          <Skeleton className="h-48 w-full" />
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Cpu className="h-5 w-5" />
              AI Provider
            </CardTitle>
            <CardDescription>
              Configure which AI service to use for problem analysis, error
              detection, and training reports.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            {/* Provider */}
            <div className="grid gap-2">
              <Label htmlFor="provider">Provider</Label>
              <Select value={provider} onValueChange={setProvider}>
                <SelectTrigger id="provider">
                  <SelectValue placeholder="Select provider" />
                </SelectTrigger>
                <SelectContent>
                  {AI_PROVIDERS.map((p) => (
                    <SelectItem key={p.value} value={p.value}>
                      {p.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* API Key */}
            <div className="grid gap-2">
              <Label htmlFor="apiKey" className="flex items-center gap-1">
                <Key className="h-4 w-4" />
                API Key
              </Label>
              <Input
                id="apiKey"
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="sk-..."
              />
              <p className="text-xs text-muted-foreground">
                Your API key is stored locally and never sent anywhere except to
                the provider you choose above.
              </p>
            </div>

            {/* Model */}
            <div className="grid gap-2">
              <Label htmlFor="model">Model</Label>
              <Input
                id="model"
                value={model}
                onChange={(e) => setModel(e.target.value)}
                placeholder="gpt-4o / claude-sonnet-4-20250514 / gemini-2.5-pro"
              />
              <p className="text-xs text-muted-foreground">
                Common: gpt-4o (OpenAI), claude-sonnet-4-20250514 (Anthropic),
                gemini-2.5-pro (Google)
              </p>
            </div>

            {/* Base URL (for custom providers) */}
            {provider === "custom" && (
              <div className="grid gap-2">
                <Label htmlFor="baseUrl" className="flex items-center gap-1">
                  <Globe className="h-4 w-4" />
                  Base URL
                </Label>
                <Input
                  id="baseUrl"
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  placeholder="https://api.deepseek.com/v1"
                />
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Error */}
      {error && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Save button */}
      <div className="flex items-center gap-4">
        <Button
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending || !provider}
        >
          <Save className="mr-2 h-4 w-4" />
          {saveMutation.isPending ? "Saving..." : "Save Settings"}
        </Button>
        {saved && (
          <span className="text-sm text-success">✓ Settings saved successfully</span>
        )}
      </div>
    </div>
  );
}
