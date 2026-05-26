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
import { Save, Key, Cpu, Globe, Languages, Monitor } from "lucide-react";

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
	const [vjudgeUsername, setVjudgeUsername] = useState("");
	const [vjudgeCookie, setVjudgeCookie] = useState("");
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
		setVjudgeUsername(settingsMap.vjudge_username ?? "");
		setVjudgeCookie(settingsMap.vjudge_cookie ?? "");
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
			await api("set_setting", {
				key: "vjudge_username",
				value: vjudgeUsername,
			});
			await api("set_setting", { key: "vjudge_cookie", value: vjudgeCookie });
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
								<Monitor className="h-5 w-5" />
								浏览器扩展（推荐）
							</CardTitle>
							<CardDescription>
								安装 ACMind 浏览器扩展后，在 VJudge
								页面点击「导入」按钮即可一键抓取题目、提交和源码，无需手动复制
								Cookie。
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-3">
							<div className="rounded-md bg-muted/50 p-3 text-sm space-y-2">
								<p className="font-medium">Chrome / Edge / Brave 安装步骤：</p>
								<ol className="list-decimal list-inside space-y-1 text-muted-foreground">
									<li>打开 <code className="text-xs bg-muted px-1 rounded">chrome://extensions</code></li>
									<li>开启右上角「开发者模式」</li>
									<li>点击「加载已解压的扩展程序」，选择 <code className="text-xs bg-muted px-1 rounded">browser-extension</code> 目录</li>
								</ol>
								<p className="font-medium mt-3">Firefox 安装步骤：</p>
								<ol className="list-decimal list-inside space-y-1 text-muted-foreground">
									<li>打开 <code className="text-xs bg-muted px-1 rounded">about:debugging#/runtime/this-firefox</code></li>
									<li>点击「临时载入附加组件」</li>
									<li>选择 <code className="text-xs bg-muted px-1 rounded">browser-extension-firefox/manifest.json</code></li>
								</ol>
								<p className="mt-2 text-xs">安装后打开 <code className="text-xs bg-muted px-1 rounded">vjudge.net</code> 并登录，页面会自动显示导入按钮。</p>
							</div>
							<p className="text-xs text-muted-foreground">
								导入服务运行在{" "}
								<code className="text-xs bg-muted px-1 rounded">
									127.0.0.1:18921
								</code>
								，扩展会自动连接。无需配置 Cookie。
							</p>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle className="flex items-center gap-2">
								<Globe className="h-5 w-5" />
								VJudge 手动配置（备选）
							</CardTitle>
							<CardDescription>
								如果不使用浏览器扩展，也可以手动填写 Cookie 来同步 VJudge
								数据。推荐优先使用浏览器扩展。
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-4">
							<div className="grid gap-2">
								<Label htmlFor="vjudgeUsername">VJudge 用户名（可选）</Label>
								<Input
									id="vjudgeUsername"
									value={vjudgeUsername}
									onChange={(e) => setVjudgeUsername(e.target.value)}
									placeholder="mengh04"
								/>
								<p className="text-xs text-muted-foreground">
									用于单题导入时自动拉取该题提交记录，不会触发批量同步。
								</p>
							</div>
							<div className="grid gap-2">
								<Label htmlFor="vjudgeCookie">VJudge Cookie（可选）</Label>
								<Input
									id="vjudgeCookie"
									type="password"
									value={vjudgeCookie}
									onChange={(e) => setVjudgeCookie(e.target.value)}
									placeholder="JSESSIONID=...; ..."
								/>
								<p className="text-xs text-muted-foreground">
									Cookie 只保存在本地。请从浏览器登录 VJudge 后复制请求
									Cookie，不要填写密码。
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
				{saved && <span className="text-sm text-success">✓ 设置已保存</span>}
			</div>
		</div>
	);
}
