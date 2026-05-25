import { Minus, Square, X } from "lucide-react";
import { useEffect, useState } from "react";

async function getAppWindow() {
	const { getCurrentWindow } = await import("@tauri-apps/api/window");
	return getCurrentWindow();
}

export function AppTitleBar() {
	const [isMaximized, setIsMaximized] = useState(false);

	useEffect(() => {
		let unlisten: (() => void) | undefined;

		getAppWindow()
			.then(async (appWindow) => {
				setIsMaximized(await appWindow.isMaximized());
				unlisten = await appWindow.onResized(async () => {
					setIsMaximized(await appWindow.isMaximized());
				});
			})
			.catch(() => {
				// Browser preview does not expose the Tauri window API.
			});

		return () => unlisten?.();
	}, []);

	const handleDrag = () => {
		getAppWindow()
			.then((appWindow) => appWindow.startDragging())
			.catch(() => {});
	};

	const minimize = () => {
		getAppWindow()
			.then((appWindow) => appWindow.minimize())
			.catch(() => {});
	};

	const toggleMaximize = () => {
		getAppWindow()
			.then(async (appWindow) => {
				await appWindow.toggleMaximize();
				setIsMaximized(await appWindow.isMaximized());
			})
			.catch(() => {});
	};

	const close = () => {
		getAppWindow()
			.then((appWindow) => appWindow.close())
			.catch(() => {});
	};

	return (
		<header
			className="flex h-11 shrink-0 items-center justify-between border-b border-border/70 bg-background/90 pl-3 shadow-[0_1px_0_rgba(255,255,255,0.65)_inset] backdrop-blur-xl"
			onMouseDown={handleDrag}
		>
			<div className="flex min-w-0 items-center gap-2">
				<div className="grid h-6 w-6 place-items-center rounded-lg bg-primary text-[11px] font-semibold text-primary-foreground shadow-sm">
					AC
				</div>
				<div className="min-w-0">
					<div className="truncate text-sm font-semibold leading-4 tracking-tight">
						ACMind
					</div>
					<div className="truncate text-[10px] leading-3 text-muted-foreground">
						算法训练工作台
					</div>
				</div>
			</div>

			<div
				className="flex h-full"
				onMouseDown={(event) => event.stopPropagation()}
			>
				<TitleBarButton label="最小化" onClick={minimize}>
					<Minus className="h-3.5 w-3.5" />
				</TitleBarButton>
				<TitleBarButton
					label={isMaximized ? "还原" : "最大化"}
					onClick={toggleMaximize}
				>
					<Square className="h-3 w-3" />
				</TitleBarButton>
				<TitleBarButton label="关闭" onClick={close} variant="close">
					<X className="h-3.5 w-3.5" />
				</TitleBarButton>
			</div>
		</header>
	);
}

interface TitleBarButtonProps {
	label: string;
	onClick: () => void;
	children: React.ReactNode;
	variant?: "default" | "close";
}

function TitleBarButton({
	label,
	onClick,
	children,
	variant = "default",
}: TitleBarButtonProps) {
	return (
		<button
			type="button"
			aria-label={label}
			title={label}
			onClick={onClick}
			className={
				variant === "close"
					? "grid w-11 place-items-center text-muted-foreground transition-colors hover:bg-destructive hover:text-destructive-foreground"
					: "grid w-11 place-items-center text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
			}
		>
			{children}
		</button>
	);
}
