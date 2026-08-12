import { createRouter, createWebHistory } from 'vue-router'

import { installRouteTransitions } from '@/lib/route-transition'
import IndexView from '@/views/IndexView.vue'
import ScopeView from '@/views/ScopeView.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'index', component: IndexView },
    { path: '/sessions/:id', name: 'session', component: ScopeView },
    { path: '/changes/:name', name: 'change', component: ScopeView },
  ],
})

installRouteTransitions(router)
