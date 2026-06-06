<script setup lang="ts">
const route = useRoute()
const email = ref('')
const password = ref('')
const feedback = ref('')
const isSubmitting = ref(false)
const { loginCustomer } = useCustomerAuth()

const submit = async () => {
  isSubmitting.value = true
  feedback.value = ''

  try {
    await loginCustomer(email.value.trim(), password.value)
    await navigateTo(typeof route.query.redirect === 'string' ? route.query.redirect : '/account')
  }
  catch (error: any) {
    feedback.value = error?.data?.error ?? 'Unable to sign in.'
  }
  finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container route-hero-inner">
        <div>
          <p class="route-kicker">Customer Login</p>
          <h1>Sign in to your Stellafrique account.</h1>
          <p class="route-copy">
            Check your saved profile and prepare for future order history once account-linked checkout lands.
          </p>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container admin-grid">
        <div class="admin-panel">
          <h2>Login</h2>
          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Email</span>
              <input v-model="email" type="email" autocomplete="username">
            </label>
            <label class="shop-field">
              <span>Password</span>
              <input v-model="password" type="password" autocomplete="current-password">
            </label>
          </div>
          <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>
          <button type="button" class="hero-cta admin-submit" :disabled="isSubmitting" @click="submit">
            {{ isSubmitting ? 'Signing in...' : 'Sign In' }}
          </button>
          <p class="route-copy">
            Need an account? <NuxtLink to="/register">Create one here</NuxtLink>.
          </p>
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
