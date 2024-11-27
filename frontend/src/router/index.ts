import { createRouter, createWebHistory, type NavigationGuardNext, type RouteLocationNormalized, type RouteLocationNormalizedLoaded } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/HomeView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('../views/LoginView.vue'),
    }
  ],
});

router.beforeEach((to: RouteLocationNormalized, from: RouteLocationNormalizedLoaded, next: NavigationGuardNext) => {
  const isAuthenticated = !!localStorage.getItem('apiToken'); 
  if (!isAuthenticated && to.name !== 'login') {
    return next({ name: 'login' });
  }

  next();
});

export default router
