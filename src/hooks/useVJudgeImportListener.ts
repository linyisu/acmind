import { useEffect, useState, useCallback, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";

interface ImportEvent {
	action: string;
	detail: string;
	timestamp: number;
}

export interface ImportNotification {
	id: number;
	message: string;
}

let nextId = 0;

/**
 * Listen for vjudge-imported Tauri events and:
 * 1. Invalidate React Query caches so the UI refreshes automatically
 * 2. Return a list of recent import notifications for display
 */
export function useVJudgeImportListener() {
	const queryClient = useQueryClient();
	const [notifications, setNotifications] = useState<ImportNotification[]>([]);
	const lastNotificationRef = useRef<{
		message: string;
		timestamp: number;
	} | null>(null);

	const dismiss = useCallback((id: number) => {
		setNotifications((prev) => prev.filter((n) => n.id !== id));
	}, []);

	useEffect(() => {
		let unlisten: (() => void) | undefined;

		async function setup() {
			try {
				const { listen } = await import("@tauri-apps/api/event");
				unlisten = await listen<ImportEvent>("vjudge-imported", (event) => {
					const { action, detail, timestamp } = event.payload;
					const message = `${action}: ${detail}`;
					const lastNotification = lastNotificationRef.current;
					if (
						lastNotification?.message === message &&
						timestamp - lastNotification.timestamp < 1000
					) {
						return;
					}

					lastNotificationRef.current = { message, timestamp };
					const id = nextId++;

					setNotifications((prev) => [...prev.slice(-4), { id, message }]);

					setTimeout(() => dismiss(id), 5000);

					queryClient.refetchQueries({
						queryKey: ["problems"],
						type: "active",
					});
					queryClient.refetchQueries({
						queryKey: ["dashboard-stats"],
						type: "active",
					});
					queryClient.refetchQueries({
						queryKey: ["submissions"],
						exact: false,
						type: "active",
					});
				});
			} catch {
				// Tauri APIs unavailable (e.g., browser dev mode) — ignore
			}
		}

		setup();

		return () => {
			unlisten?.();
		};
	}, [queryClient, dismiss]);

	return { notifications, dismiss };
}
