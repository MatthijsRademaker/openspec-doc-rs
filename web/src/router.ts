import { createRouter, createWebHistory } from 'vue-router'

import IndexView from '@/views/IndexView.vue'

// Only `/` exists: the session and change pages are served by the Rust server
// today and become Vue routes in `migrate-dashboard-review-to-vue`, when their
// views and JSON endpoints exist. A placeholder route here would render fake
// scope data under a real URL, which is worse than the server's 404.
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: '/', name: 'index', component: IndexView }],
})
