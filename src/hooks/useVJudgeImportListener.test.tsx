import {
	QueryClient,
	QueryClientProvider,
	useQuery,
} from "@tanstack/react-query";
import { act, render, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useVJudgeImportListener } from "./useVJudgeImportListener";

const listenMock = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
	listen: listenMock,
}));

function ActiveProblemsQuery({ queryFn }: { queryFn: () => Promise<unknown> }) {
	useQuery({ queryKey: ["problems"], queryFn });
	return null;
}

function TestComponent({ queryFn }: { queryFn?: () => Promise<unknown> }) {
	useVJudgeImportListener();
	return queryFn ? <ActiveProblemsQuery queryFn={queryFn} /> : null;
}

function renderWithClient(
	client: QueryClient,
	queryFn?: () => Promise<unknown>,
) {
	return render(<TestComponent queryFn={queryFn} />, {
		wrapper: ({ children }: { children: ReactNode }) => (
			<QueryClientProvider client={client}>{children}</QueryClientProvider>
		),
	});
}

describe("useVJudgeImportListener", () => {
	beforeEach(() => {
		listenMock.mockReset();
		listenMock.mockResolvedValue(vi.fn());
	});

	afterEach(() => {
		vi.clearAllTimers();
	});

	it("refreshes active imported data without refetching unrelated pages", async () => {
		let handler: ((event: { payload: unknown }) => void) | undefined;
		listenMock.mockImplementation(async (_eventName, callback) => {
			handler = callback;
			return vi.fn();
		});

		const problemsQuery = vi.fn(async () => []);
		const client = new QueryClient({
			defaultOptions: { queries: { retry: false } },
		});
		client.setQueryData(["dashboard-stats"], { total_problems: 0 });
		client.setQueryData(["reports"], []);

		renderWithClient(client, problemsQuery);

		await waitFor(() => expect(handler).toBeDefined());
		await waitFor(() => expect(problemsQuery).toHaveBeenCalledTimes(1));

		act(() => {
			handler?.({
				payload: {
					action: "submissions",
					detail: "1 imported",
					timestamp: Date.now(),
				},
			});
		});

		await waitFor(() => expect(problemsQuery).toHaveBeenCalledTimes(2));
		expect(client.getQueryState(["dashboard-stats"])?.fetchStatus).toBe("idle");
		expect(client.getQueryState(["reports"])?.isInvalidated).toBe(false);
	});
});
