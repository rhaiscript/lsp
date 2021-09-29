import "@quasar/extras/roboto-font/roboto-font.css";
import "@quasar/extras/mdi-v5/mdi-v5.css";
import "quasar/dist/quasar.prod.css";

import { Quasar, ClosePopup, Dark, QuasarPluginOptions, Notify, Meta, Dialog, Ripple, Loading } from "quasar";

import iconSet from "quasar/icon-set/mdi-v5";
import lang from "quasar/lang/hu";
import { Plugin } from "vue";

import components from "./components";

const plugin: Plugin = {
  install: app => {
    const options: QuasarPluginOptions = {
      lang: lang as any,
      iconSet: iconSet as any,
      directives: { ClosePopup, Ripple },
      plugins: {
        Dark,
        Notify,
        Meta,
        Dialog,
        Loading,
      },
      components,
      config: {
        brand: {
          primary: "#55BB2A",
          secondary: "#BB482A",
          positive: "#55BB2A",
          info: "#2977DB",
          negative: "#DB3429",
        },
        globalProperties: {},
      },
    };

    app.use(Quasar, options);
  },
};

export default plugin;
