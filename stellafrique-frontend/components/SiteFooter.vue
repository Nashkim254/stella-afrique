<script setup lang="ts">
const {
  storeName,
  footerGroups,
} = useStorefrontContent()

const signupEmail = ref('')

const goToRegister = async () => {
  const email = signupEmail.value.trim()
  const query = email ? { email } : undefined
  await navigateTo({
    path: '/register',
    query,
  })
}
</script>

<template>
  <footer class="site-footer">
    <div class="container footer-grid">
      <div>
        <h3>{{ storeName }}</h3>
        <form class="footer-signup" @submit.prevent="goToRegister">
          <input v-model="signupEmail" type="email" placeholder="Enter Email Address" autocomplete="email">
          <button type="submit">Sign Up</button>
        </form>
        <p>17 Riverside Road, Nairobi, Kenya</p>
      </div>

      <div v-for="group in footerGroups" :key="group.title">
        <h4>{{ group.title }}</h4>
        <NuxtLink v-for="link in group.links" :key="link.to" :to="link.to">
          {{ link.label }}
        </NuxtLink>
      </div>
    </div>
  </footer>
</template>
