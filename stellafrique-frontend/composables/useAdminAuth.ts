export const useAdminAuth = () => {
  const config = useRuntimeConfig()
  const session = useState<{
    authenticated: boolean
    user: {
      id: string
      email: string
      full_name: string
      role: string
    } | null
    checked: boolean
  }>('admin-session', () => ({
    authenticated: false,
    user: null,
    checked: false,
  }))

  const refreshAdminSession = async (force = false) => {
    if (session.value.checked && !force) {
      return session.value
    }

    try {
      const response = await $fetch<{
        authenticated: boolean
        user?: {
          id: string
          email: string
          full_name: string
          role: string
        } | null
      }>('/admin/auth/session', {
        baseURL: config.public.apiBaseUrl,
        credentials: 'include',
      })

      session.value = {
        authenticated: response.authenticated,
        user: response.user ?? null,
        checked: true,
      }
    }
    catch {
      session.value = {
        authenticated: false,
        user: null,
        checked: true,
      }
    }

    return session.value
  }

  const loginAdmin = async (email: string, password: string) => {
    const response = await $fetch<{
      authenticated: boolean
      user?: {
        id: string
        email: string
        full_name: string
        role: string
      } | null
    }>('/admin/auth/login', {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        email,
        password,
      },
    })

    session.value = {
      authenticated: response.authenticated,
      user: response.user ?? null,
      checked: true,
    }

    return response
  }

  const logoutAdmin = async () => {
    await $fetch('/admin/auth/logout', {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
    })

    session.value = {
      authenticated: false,
      user: null,
      checked: true,
    }
  }

  return {
    session: computed(() => session.value),
    adminUser: computed(() => session.value.user),
    adminEmail: computed(() => session.value.user?.email ?? null),
    adminName: computed(() => session.value.user?.full_name ?? null),
    adminRole: computed(() => session.value.user?.role ?? null),
    isAdminAuthenticated: computed(() => session.value.authenticated),
    refreshAdminSession,
    loginAdmin,
    logoutAdmin,
  }
}
