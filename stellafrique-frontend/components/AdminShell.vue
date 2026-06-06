<script setup lang="ts">
const props = defineProps<{
  title: string
  description: string
  kicker?: string
}>()

const route = useRoute()
const { adminName, adminEmail, adminRole, logoutAdmin } = useAdminAuth()

const adminLinks = computed(() => [
  { label: 'Dashboard', to: '/admin' },
  { label: 'Orders', to: '/admin/orders' },
  { label: 'Catalog', to: '/admin/catalog' },
  { label: 'Inventory', to: '/admin/inventory' },
  ...(['owner', 'admin'].includes((adminRole.value ?? '').toLowerCase())
    ? [{ label: 'Withdrawals', to: '/admin/withdrawals' }]
    : []),
  ...((adminRole.value ?? '').toLowerCase() === 'owner'
    ? [{ label: 'Staff', to: '/admin/staff' }]
    : []),
])

const handleLogout = async () => {
  await logoutAdmin()
  await navigateTo('/admin/login')
}
</script>

<template>
  <main class="admin-shell">
    <aside class="admin-sidebar">
      <NuxtLink to="/admin" class="admin-brand">Stellafrique Admin</NuxtLink>
      <p class="admin-sidebar-copy">Operations, sales, catalogue control, and stock visibility in one place.</p>

      <nav class="admin-nav">
        <NuxtLink
          v-for="link in adminLinks"
          :key="link.to"
          :to="link.to"
          class="admin-nav-link"
          :class="{ 'is-active': route.path === link.to || (link.to !== '/admin' && route.path.startsWith(link.to)) }"
        >
          {{ link.label }}
        </NuxtLink>
      </nav>

      <div class="admin-sidebar-card">
        <span class="admin-sidebar-label">Signed in as</span>
        <strong>{{ adminName || adminEmail || 'Administrator' }}</strong>
        <span class="admin-sidebar-label">{{ adminRole || 'staff' }}</span>
        <button type="button" class="secondary-link admin-sidebar-button" @click="handleLogout">
          Sign Out
        </button>
      </div>
    </aside>

    <div class="admin-main">
      <section class="admin-hero">
        <p class="route-kicker">{{ props.kicker || 'Admin' }}</p>
        <h1>{{ props.title }}</h1>
        <p class="route-copy">{{ props.description }}</p>
      </section>

      <slot />
    </div>
  </main>
</template>
