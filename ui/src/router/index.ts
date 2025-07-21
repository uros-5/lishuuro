import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/components/views/HomeView.vue'),
    },
    {
      name: 'game',
      path: '/game/:id',
      component: () => import('@/components/views/GameView.vue'),
    },
    {
      name: 'profile',
      path: '/@/:id',
      component: () => import('@/components/views/UserProfile.vue'),
    },
    {
      name: 'tv',
      path: '/tv',
      component: () => import('@/components/views/TvView.vue'),
    },
    {
      name: 'rules',
      path: '/how-to-play-shuuro',
      component: () => import('@/components/views/HowToPlay.vue'),
    },
    {
      name: 'logged',
      path: '/logged',
      component: () => import('@/components/views/Logged.vue'),
    },
    {
      name: "custom-piece",
      path: "/custom-piece",
      component: () => import("@/components/views/CustomPieces.vue")
    }
  ],
})

export default router
