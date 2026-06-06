export default defineNuxtRouteMiddleware(async (to) => {
  if (to.path !== '/account') {
    return
  }

  if (import.meta.server) {
    return
  }

  const { isCustomerAuthenticated, refreshCustomerSession } = useCustomerAuth()
  await refreshCustomerSession()

  if (!isCustomerAuthenticated.value) {
    return navigateTo(`/login?redirect=${encodeURIComponent(to.fullPath)}`)
  }
})
