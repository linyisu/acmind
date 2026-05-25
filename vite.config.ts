import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
	plugins: [react(), tailwindcss()],

	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),
		},
	},

	// Prevent vite from obscuring Rust errors
	clearScreen: false,

	server: {
		// Tauri expects a fixed port; fail if that port is not available
		strictPort: true,
	},
});
