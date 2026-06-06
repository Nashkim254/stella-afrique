<script setup lang="ts">
const route = useRoute()
const fullName = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')
const feedback = ref('')
const isSubmitting = ref(false)
const { registerCustomer } = useCustomerAuth()

const submit = async () => {
  const trimmedFullName = fullName.value.trim()
  const trimmedEmail = email.value.trim()
  isSubmitting.value = true
  feedback.value = ''

  if (!trimmedFullName || !trimmedEmail || !password.value || !confirmPassword.value) {
    feedback.value = 'Full name, email, password, and confirm password are required.'
    isSubmitting.value = false
    return
  }

  if (password.value.length < 8) {
    feedback.value = 'Password must be at least 8 characters.'
    isSubmitting.value = false
    return
  }

  if (password.value !== confirmPassword.value) {
    feedback.value = 'Password confirmation does not match.'
    isSubmitting.value = false
    return
  }

  try {
    await registerCustomer(trimmedFullName, trimmedEmail, password.value)
    await navigateTo(typeof route.query.redirect === 'string' ? route.query.redirect : '/account')
  }
  catch (error: any) {
    feedback.value = error?.data?.error ?? 'Unable to create account.'
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
          <p class="route-kicker">Create Account</p>
          <h1>Register for a Stellafrique account.</h1>
          <p class="route-copy">
            Save your details now so the storefront can grow into account-linked checkout and order history.
          </p>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container admin-grid">
        <div class="admin-panel">
          <h2>Register</h2>
          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Full Name</span>
              <input v-model="fullName" type="text" autocomplete="name">
            </label>
            <label class="shop-field">
              <span>Email</span>
              <input v-model="email" type="email" autocomplete="email">
            </label>
            <label class="shop-field">
              <span>Password</span>
              <input v-model="password" type="password" autocomplete="new-password">
            </label>
            <label class="shop-field">
              <span>Confirm Password</span>
              <input v-model="confirmPassword" type="password" autocomplete="new-password">
            </label>
          </div>
          <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>
          <button type="button" class="hero-cta admin-submit" :disabled="isSubmitting" @click="submit">
            {{ isSubmitting ? 'Creating account...' : 'Create Account' }}
          </button>
          <p class="route-copy">
            Already registered? <NuxtLink to="/login">Sign in here</NuxtLink>.
          </p>
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
