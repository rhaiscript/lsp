import { defineConfig, UserConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import yaml from "@rollup/plugin-yaml";

// https://vitejs.dev/config/
export default defineConfig(() => {
  const config: UserConfig = {
    plugins: [vue(), yaml()],
    build: {
      chunkSizeWarningLimit: 2000,
    },
    resolve: {
      alias: [{ find: "@", replacement: path.resolve(__dirname, "src") }],
    },
  };

  return config
});
