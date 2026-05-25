import { describe, it, expect } from "vitest";
import { cn } from "./utils";

describe("cn (className merge utility)", () => {
	it("should merge multiple static class names", () => {
		expect(cn("text-red-500", "bg-blue-500")).toBe("text-red-500 bg-blue-500");
	});

	it("should filter out falsy values", () => {
		expect(cn("foo", false, null, undefined, "bar")).toBe("foo bar");
		expect(cn("foo", 0, "", "bar")).toBe("foo bar");
	});

	it("should handle conditional classes with objects", () => {
		expect(cn("base", { active: true, disabled: false })).toBe("base active");
	});

	it("should return empty string when no classes provided", () => {
		expect(cn()).toBe("");
		expect(cn(false, null, undefined)).toBe("");
	});

	it("should merge tailwind-merge compatible classes", () => {
		// twMerge resolves conflicting tailwind utilities
		expect(cn("p-4", "p-2")).toBe("p-2");
		expect(cn("text-red-500", "text-blue-500")).toBe("text-blue-500");
	});

	it("should handle arrays of classes", () => {
		expect(cn(["foo", "bar"])).toBe("foo bar");
		expect(cn("base", ["conditional", "extra"])).toBe("base conditional extra");
	});
});
