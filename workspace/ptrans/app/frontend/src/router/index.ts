import { createRouter, createWebHistory } from "vue-router";
import Home from "@/views/Home.vue";
import Pool from "@/views/Pool.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "home",
      component: Home,
    },
    {
      path: "/pool",
      name: "pool",
      component: Pool,
    },
  ],
});

export default router;
