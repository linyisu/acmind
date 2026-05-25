import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
	Card,
	CardContent,
	CardHeader,
	CardTitle,
	CardDescription,
} from "@/components/ui/card";
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
import i18n from "@/lib/i18n";
import { Save, Key, Cpu, Globe, Languages } from "lucide-react";

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

const LOCALES = [
	{ value: "zh", label: "中文" },
	{ value: "en", label: "English" },
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
	const [locale, setLocale] = useState("zh");
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
		setLocale(settingsMap.app_locale ?? "zh");
		setInitialized(true);
	}

	const saveMutation = useMutation({
		mutationFn: async () => {
			setError("");
			await api("set_setting", { key: "ai_provider", value: provider });
			await api("set_setting", { key: "ai_api_key", value: apiKey });
			await api("set_setting", { key: "ai_model", value: model });
			await api("set_setting", { key: "ai_base_url", value: baseUrl });
			await api("set_setting", { key: "app_locale", value: locale });
			await i18n.changeLanguage(locale);
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["settings"] });
			setSaved(true);
			setTimeout(() => setSaved(false), 3000);
		},
		onError: (err) => {
			setError(err instanceof Error ? err.message : "保存失败");
		},
	});

	return (
		<div className="space-y-6 max-w-2xl">
			<div>
				<h1 className="text-2xl font-bold">设置</h1>
				<p className="text-muted-foreground">
					配置语言和 AI 提供商，用于自动分析与报告生成
				</p>
			</div>

			{isLoading ? (
				<div className="space-y-4">
					<Skeleton className="h-48 w-full" />
				</div>
			) : (
				<>
					<Card className="mb-6">
					<CardHeader>
						<CardTitle className="flex items-center gap-2">
							<Languages className="h-5 w-5" />
							语言
						</CardTitle>
						<CardDescription>
							选择界面语言，同时影响 AI 分析提示词和输出语言。
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="grid gap-2">
							<Label htmlFor="locale">应用语言</Label>
							<Select value={locale} onValueChange={setLocale}>
								<SelectTrigger id="locale">
									<SelectValue placeholder="选择语言" />
								</SelectTrigger>
								<SelectContent>
									{LOCALES.map((item) => (
										<SelectItem key={item.value} value={item.value}>
											{item.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
							<p className="text-xs text-muted-foreground">
								保存后，后端 AI 提示词会使用同一语言。
							</p>
						</div>
					</CardContent>
					</Card>

					<Card>
					<CardHeader>
						<CardTitle className="flex items-center gap-2">
							<Cpu className="h-5 w-5" />
							AI 提供商
						</CardTitle>
						<CardDescription>
							配置用于题目分析、错误检测和训练报告的 AI 服务。
						</CardDescription>
					</CardHeader>
					<CardContent className="space-y-6">
						{/* Provider */}
						<div className="grid gap-2">
							<Label htmlFor="provider">提供商</Label>
							<Select value={provider} onValueChange={setProvider}>
								<SelectTrigger id="provider">
									<SelectValue placeholder="选择提供商" />
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
								你的 API Key 只会保存在本地，并且只发送给上面选择的提供商。
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
								常用：gpt-4o (OpenAI)、claude-sonnet-4-20250514 (Anthropic)、
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
				</>
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
					{saveMutation.isPending ? "保存中..." : "保存设置"}
				</Button>
				{saved && (
					<span className="text-sm text-success">✓ 设置已保存</span>
				)}
			</div>
		</div>
	);
}
