export const useCustomerAuth = () => {
  const config = useRuntimeConfig()
  const session = useState<{
    authenticated: boolean
    checked: boolean
    user: null | {
      id: string
      full_name: string
      email: string
      phone?: string | null
      address_line1?: string | null
      address_line2?: string | null
      city?: string | null
      region?: string | null
      postal_code?: string | null
      country?: string | null
      created_at: string
    }
  }>('customer-session', () => ({
    authenticated: false,
    checked: false,
    user: null,
  }))

  const refreshCustomerSession = async (force = false) => {
    if (session.value.checked && !force) {
      return session.value
    }

    try {
      const response = await $fetch<{
        authenticated: boolean
        user?: {
          id: string
          full_name: string
          email: string
          phone?: string | null
          address_line1?: string | null
          address_line2?: string | null
          city?: string | null
          region?: string | null
          postal_code?: string | null
          country?: string | null
          created_at: string
        } | null
      }>('/auth/session', {
        baseURL: config.public.apiBaseUrl,
        credentials: 'include',
      })

      session.value = {
        authenticated: response.authenticated,
        checked: true,
        user: response.user ?? null,
      }
    }
    catch {
      session.value = {
        authenticated: false,
        checked: true,
        user: null,
      }
    }

    return session.value
  }

  const registerCustomer = async (fullName: string, email: string, password: string) => {
    const response = await $fetch('/auth/register', {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        full_name: fullName,
        email,
        password,
      },
    })

    await refreshCustomerSession(true)
    return response
  }

  const loginCustomer = async (email: string, password: string) => {
    const response = await $fetch('/auth/login', {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        email,
        password,
      },
    })

    await refreshCustomerSession(true)
    return response
  }

  const logoutCustomer = async () => {
    await $fetch('/auth/logout', {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
    })

    session.value = {
      authenticated: false,
      checked: true,
      user: null,
    }
  }

  return {
    session: computed(() => session.value),
    currentCustomer: computed(() => session.value.user),
    isCustomerAuthenticated: computed(() => session.value.authenticated),
    refreshCustomerSession,
    registerCustomer,
    loginCustomer,
    logoutCustomer,
  }
}
