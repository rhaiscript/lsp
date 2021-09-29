import { defineStore } from "pinia";

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface MainState {}

export const useMainStore = defineStore({
  id: "main",
  state: (): MainState => ({}),
});
