import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Badge } from "./badge";

describe("Badge component", () => {
	it("should render text content", () => {
		render(<Badge>AC</Badge>);
		expect(screen.getByText("AC")).toBeInTheDocument();
	});

	it("should apply default variant", () => {
		render(<Badge>Default</Badge>);
		const badge = screen.getByText("Default");
		expect(badge).toHaveClass("bg-primary");
	});

	it("should apply success variant", () => {
		render(<Badge variant="success">AC</Badge>);
		const badge = screen.getByText("AC");
		expect(badge).toHaveClass("bg-success");
		expect(badge).toHaveClass("text-success-foreground");
	});

	it("should apply error variant", () => {
		render(<Badge variant="error">WA</Badge>);
		const badge = screen.getByText("WA");
		expect(badge).toHaveClass("bg-error");
	});

	it("should apply warning variant", () => {
		render(<Badge variant="warning">TLE</Badge>);
		const badge = screen.getByText("TLE");
		expect(badge).toHaveClass("bg-warning");
	});

	it("should apply custom className", () => {
		render(<Badge className="custom">Custom</Badge>);
		const badge = screen.getByText("Custom");
		expect(badge).toHaveClass("custom");
	});
});
