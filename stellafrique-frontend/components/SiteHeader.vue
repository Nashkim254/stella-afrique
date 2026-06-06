<script setup lang="ts">
const {
  storeName,
  navLinks,
} = useStorefrontContent()
const { itemCount } = useCart()
const { isAdminAuthenticated, adminRole, logoutAdmin, refreshAdminSession } = useAdminAuth()
const { isCustomerAuthenticated, currentCustomer, logoutCustomer, refreshCustomerSession } = useCustomerAuth()

onMounted(() => {
  refreshAdminSession()
  refreshCustomerSession()
})
</script>

<template>
  <section class="topbar">
    <div class="container topbar-inner">
      <div class="topbar-copy">
        <span>hello@stellafrique.com</span>
        <span>+254 712 345 678</span>
      </div>
      <div class="topbar-copy">
        <span>English</span>
        <span>KES</span>
        <template v-if="isCustomerAuthenticated">
          <NuxtLink to="/account">{{ currentCustomer?.full_name || 'My Account' }}</NuxtLink>
          <button type="button" class="topbar-button" @click="logoutCustomer">Customer Logout</button>
        </template>
        <template v-else>
          <NuxtLink to="/login">Login</NuxtLink>
          <NuxtLink to="/register">Register</NuxtLink>
        </template>
        <NuxtLink to="/collections/sale">Wishlist</NuxtLink>
        <template v-if="isAdminAuthenticated">
          <NuxtLink to="/admin">Admin {{ adminRole ? `· ${adminRole}` : '' }}</NuxtLink>
          <button type="button" class="topbar-button" @click="logoutAdmin">Admin Logout</button>
        </template>
      </div>
    </div>
  </section>

  <header class="navbar">
    <div class="container navbar-inner">
      <NuxtLink to="/" class="brand">{{ storeName }}</NuxtLink>
      <nav class="nav-links">
        <NuxtLink v-for="link in navLinks" :key="link.to" :to="link.to">
          {{ link.label }}
        </NuxtLink>
        <NuxtLink to="/cart" class="cart-link">
          Cart
          <span v-if="itemCount" class="cart-count">{{ itemCount }}</span>
        </NuxtLink>
      </nav>
      <div class="search-shell">
        <input type="text" placeholder="Search for dresses, knitwear, sets">
        <NuxtLink to="/shop" class="search-button">Search</NuxtLink>
      </div>
    </div>
  </header>
</template>
