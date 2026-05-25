import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Card, CardHeader, CardTitle, CardContent } from "./card";

describe("Card components", () => {
	it("should render a card with header and content", () => {
		render(
			<Card data-testid="card">
				<CardHeader>
					<CardTitle>Problem Stats</CardTitle>
				</CardHeader>
				<CardContent>
					<p>12 problems, 85% AC rate</p>
				</CardContent>
			</Card>,
		);

		expect(screen.getByTestId("card")).toBeInTheDocument();
		expect(screen.getByText("Problem Stats")).toBeInTheDocument();
		expect(screen.getByText("12 problems, 85% AC rate")).toBeInTheDocument();
	});

	it("should render empty card", () => {
		render(<Card data-testid="empty" />);
		expect(screen.getByTestId("empty")).toBeInTheDocument();
	});

	it("should apply custom className to card", () => {
		render(<Card className="border-dashed" data-testid="styled" />);
		expect(screen.getByTestId("styled")).toHaveClass("border-dashed");
	});

	it("should render CardHeader with spacing", () => {
		render(
			<Card>
				<CardHeader>
					<CardTitle>Header</CardTitle>
				</CardHeader>
			</Card>,
		);
		expect(screen.getByText("Header")).toBeInTheDocument();
	});
});
