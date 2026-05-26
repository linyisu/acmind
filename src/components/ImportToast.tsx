import type { ImportNotification } from "@/hooks/useVJudgeImportListener";

export function ImportToast({
	notifications,
	dismiss,
}: {
	notifications: ImportNotification[];
	dismiss: (id: number) => void;
}) {
	if (notifications.length === 0) return null;

	return (
		<div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
			{notifications.map((n) => (
				<div
					key={n.id}
					className="animate-in slide-in-from-right rounded-lg border border-success/30 bg-success/10 px-4 py-2 text-sm text-success shadow-lg backdrop-blur cursor-pointer"
					onClick={() => dismiss(n.id)}
					onKeyDown={(e) => {
						if (e.key === "Enter" || e.key === " ") dismiss(n.id);
					}}
					role="button"
					tabIndex={0}
				>
					{n.message}
				</div>
			))}
		</div>
	);
}
