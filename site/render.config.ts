import { defaultServerConditions, defineConfig } from "vite";

export default defineConfig({
  publicDir: false,
  ssr: { resolve: { conditions: ["source", ...defaultServerConditions], externalConditions: ["source"] }, noExternal: [] },
  build: {
    ssr: "src/render.tsx",
    outDir: ".generated",
    emptyOutDir: true,
    rollupOptions: { output: { entryFileNames: "render.mjs" } },
  },
});
