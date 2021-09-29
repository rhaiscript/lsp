import { createApp } from "vue";
import App from "./App.vue";

import quasar from "./plugins/quasar";
import i18n from "./plugins/i18n";
import router from "./plugins/router";
import store from "./plugins/store";

import "./styles/main.scss";

import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import JsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";

(self as any).MonacoEnvironment = {
  getWorker(_: any, label: any): Worker {
    if (label === "json") {
      return new JsonWorker();
    }

    return new EditorWorker();
  },
} as any;

createApp(App).use(quasar).use(i18n).use(router).use(store).mount("#app");
