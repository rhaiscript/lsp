import { createRouter, createWebHashHistory, RouteRecordRaw } from "vue-router";
import Home from "@/views/Home.vue";

declare module "vue-router" {
  interface RouteMeta {
    /** Types for route metadata. */
    example?: boolean;
  }
}

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    component: Home,
    name: "Home",
  },
  {
    path: "/:catchAll(.*)",
    redirect: { name: "Home" },
  },
];

const router = createRouter({ history: createWebHashHistory(), routes });

export default router;
