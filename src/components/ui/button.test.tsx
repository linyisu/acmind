import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Button } from "./button";

describe("Button component", () => {
	it("should render with default variant", () => {
		render(<Button>Click me</Button>);
		const button = screen.getByRole("button", { name: "Click me" });
		expect(button).toBeInTheDocument();
		expect(button).toHaveClass("bg-primary");
	});

	it("should render with variant classes", () => {
		render(<Button variant="outline">Outline</Button>);
		const button = screen.getByRole("button", { name: "Outline" });
		expect(button).toHaveClass("border");
		expect(button).toHaveClass("bg-background");
	});

	it("should render with size classes", () => {
		render(<Button size="sm">Small</Button>);
		const button = screen.getByRole("button", { name: "Small" });
		expect(button).toHaveClass("h-9");
	});

	it("should render as child element when asChild is used", () => {
		render(
			<Button asChild>
				<a href="/">Link</a>
			</Button>,
		);
		const link = screen.getByRole("link", { name: "Link" });
		expect(link).toBeInTheDocument();
		expect(link).toHaveAttribute("href", "/");
	});

	it("should apply custom className", () => {
		render(<Button className="custom">Custom</Button>);
		const button = screen.getByRole("button", { name: "Custom" });
		expect(button).toHaveClass("custom");
	});

	it("should handle disabled state", () => {
		render(<Button disabled>Disabled</Button>);
		const button = screen.getByRole("button", { name: "Disabled" });
		expect(button).toBeDisabled();
	});
});
