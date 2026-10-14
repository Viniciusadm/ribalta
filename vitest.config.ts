import react from "@vitejs/plugin-react";
import { defaultClientConditions } from "vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: { conditions: ["source", ...defaultClientConditions] },
  plugins: [react()],
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.{ts,tsx}"],
  },
});
