import { Quasar } from "quasar";
import { Plugin } from "vue";
import { createI18n } from "vue-i18n";

import huHu from "../lang/hu-hu.yaml";
import enGb from "../lang/en-gb.yaml";

import quasarEnGb from "quasar/lang/en-GB";
import quasarHu from "quasar/lang/hu";
import { unreachable } from "@/util";

export const supportedLocales = <const>["hu-hu", "en-gb"];

export type SupportedLocale = typeof supportedLocales[keyof typeof supportedLocales];

export const i18n = createI18n({
  locale: "en-gb",
  fallbackLocale: "en-gb",
  messages: {
    "hu-hu": huHu,
    "en-gb": enGb,
  },
});

// Do not make this sync as this might become async in the future.
export async function setLocale(locale: SupportedLocale): Promise<void> {
  if (i18n.global.locale === locale) {
    return;
  }

  switch (locale) {
    case "hu-hu":
      Quasar.lang = quasarHu as any;
      break;
    case "en-gb":
      Quasar.lang = quasarEnGb as any;
      break;
    default:
      unreachable();
  }

  i18n.global.locale = locale;
}

const plugin: Plugin = {
  install: app => {
    app.use(i18n);
  },
};

export default plugin;
