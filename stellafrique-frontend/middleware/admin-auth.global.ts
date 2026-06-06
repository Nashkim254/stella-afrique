export default defineNuxtRouteMiddleware(async (to) => {
  if (!to.path.startsWith('/admin') || to.path === '/admin/login') {
    return
  }

  if (import.meta.server) {
    return
  }

  const { isAdminAuthenticated, refreshAdminSession } = useAdminAuth()
  await refreshAdminSession()

  if (!isAdminAuthenticated.value) {
    return navigateTo(`/admin/login?redirect=${encodeURIComponent(to.fullPath)}`)
  }
})
