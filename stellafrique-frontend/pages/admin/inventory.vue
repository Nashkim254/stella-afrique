<script setup lang="ts">
const config = useRuntimeConfig()

type InventoryRecord = {
  product_name: string
  product_slug: string
  variant_id: string
  variant_name: string
  sku: string
  size?: string | null
  color?: string | null
  stock_quantity: number
  is_active: boolean
  latest_event?: {
    event_type: string
    actor: string
    message: string
    reason: string
    created_at: string
  } | null
}

type InventoryEventRecord = {
  event_type: string
  actor: string
  message: string
  reason: string
  previous_stock_quantity?: number | null
  next_stock_quantity?: number | null
  previous_is_active?: boolean | null
  next_is_active?: boolean | null
  created_at: string
}

const search = ref('')
const feedback = ref('')
const savingId = ref('')
const loadingHistoryId = ref('')
const adminFetchOptions = {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include' as const,
}

const { data: inventory } = await useFetch<InventoryRecord[]>('/admin/inventory', {
  ...adminFetchOptions,
  default: () => [],
  server: false,
})

const stockDrafts = reactive<Record<string, string>>({})
const activeDrafts = reactive<Record<string, boolean>>({})
const reasonDrafts = reactive<Record<string, string>>({})
const openHistory = reactive<Record<string, boolean>>({})
const historyRecords = reactive<Record<string, InventoryEventRecord[]>>({})
const historyErrors = reactive<Record<string, string>>({})

watch(
  inventory,
  (value) => {
    for (const item of value) {
      stockDrafts[item.variant_id] = String(item.stock_quantity)
      activeDrafts[item.variant_id] = item.is_active
      reasonDrafts[item.variant_id] = ''
      openHistory[item.variant_id] = openHistory[item.variant_id] ?? false
    }
  },
  { immediate: true },
)

const filteredInventory = computed(() => {
  const term = search.value.trim().toLowerCase()
  if (!term) {
    return inventory.value
  }

  return inventory.value.filter((item) =>
    [item.product_name, item.variant_name, item.sku, item.size, item.color]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(term)),
  )
})

const saveInventory = async (variantId: string) => {
  savingId.value = variantId
  feedback.value = ''

  try {
    const updated = await $fetch<InventoryRecord>(`/admin/inventory/${variantId}`, {
      method: 'PATCH',
      ...adminFetchOptions,
      body: {
        stock_quantity: Number(stockDrafts[variantId]),
        is_active: activeDrafts[variantId],
        adjustment_reason: reasonDrafts[variantId],
      },
    })

    const index = inventory.value.findIndex((item) => item.variant_id === variantId)
    if (index >= 0) {
      inventory.value[index] = updated
    }
    stockDrafts[variantId] = String(updated.stock_quantity)
    activeDrafts[variantId] = updated.is_active
    reasonDrafts[variantId] = ''
    historyRecords[variantId] = []
    openHistory[variantId] = false
    feedback.value = `${updated.sku} inventory updated.`
  }
  finally {
    savingId.value = ''
  }
}

const toggleHistory = async (variantId: string) => {
  if (openHistory[variantId]) {
    openHistory[variantId] = false
    return
  }

  if (!historyRecords[variantId]) {
    loadingHistoryId.value = variantId
    historyErrors[variantId] = ''

    try {
      historyRecords[variantId] = await $fetch<InventoryEventRecord[]>(`/admin/inventory/${variantId}`, {
        ...adminFetchOptions,
      })
    }
    catch (error) {
      historyErrors[variantId] = error instanceof Error ? error.message : 'Unable to load history.'
    }
    finally {
      loadingHistoryId.value = ''
    }
  }

  openHistory[variantId] = true
}
</script>

<template>
  <AdminShell
    title="Inventory overview for every variant."
    description="Track low-stock sizes, inactive variants, and SKU-level availability without opening each product."
  >
    <section class="admin-content-section">
      <div class="admin-panel">
        <div class="admin-section-heading">
          <div>
            <h2>Inventory</h2>
            <p>{{ filteredInventory.length }} variant records</p>
          </div>
          <label class="shop-field inventory-search">
            <span>Search</span>
            <input v-model="search" type="text" placeholder="Search by product, SKU, size, or colour">
          </label>
        </div>
        <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>

        <div class="admin-table inventory-table">
          <article v-for="item in filteredInventory" :key="item.variant_id" class="admin-row inventory-row">
            <div>
              <strong>{{ item.product_name }}</strong>
              <p>{{ item.variant_name }} · {{ item.sku }}</p>
              <p v-if="item.size || item.color">
                <template v-if="item.size">Size: {{ item.size }}</template>
                <template v-if="item.size && item.color"> · </template>
                <template v-if="item.color">Colour: {{ item.color }}</template>
              </p>
            </div>
            <div class="inventory-meta">
              <span
                class="admin-status-pill"
                :class="{
                  'admin-status-pill-danger': item.stock_quantity <= 2,
                  'admin-status-pill-warning': item.stock_quantity > 2 && item.stock_quantity <= 5,
                }"
              >
                {{ item.stock_quantity }} in stock
              </span>
              <span class="admin-status-pill admin-status-pill-secondary">
                {{ item.is_active ? 'active' : 'inactive' }}
              </span>
            </div>
            <div class="inventory-controls">
              <label class="shop-field">
                <span>Stock</span>
                <input v-model="stockDrafts[item.variant_id]" type="number" min="0">
              </label>
              <label class="shop-field admin-checkbox inventory-active-toggle">
                <span>Active</span>
                <input v-model="activeDrafts[item.variant_id]" type="checkbox">
              </label>
              <label class="shop-field inventory-reason">
                <span>Reason</span>
                <textarea
                  v-model="reasonDrafts[item.variant_id]"
                  rows="3"
                  placeholder="Why are you changing this stock level or availability?"
                />
              </label>
              <button
                type="button"
                class="hero-cta admin-submit inventory-save"
                :disabled="savingId === item.variant_id"
                @click="saveInventory(item.variant_id)"
              >
                {{ savingId === item.variant_id ? 'Saving...' : 'Save' }}
              </button>
              <button
                type="button"
                class="secondary-link inventory-history-toggle"
                :disabled="loadingHistoryId === item.variant_id"
                @click="toggleHistory(item.variant_id)"
              >
                {{ loadingHistoryId === item.variant_id ? 'Loading...' : openHistory[item.variant_id] ? 'Hide History' : 'View History' }}
              </button>
            </div>
            <div v-if="item.latest_event" class="inventory-latest-event">
              <strong>Latest adjustment</strong>
              <p>{{ item.latest_event.message }}</p>
              <p>Reason: {{ item.latest_event.reason }}</p>
              <p>{{ new Date(item.latest_event.created_at).toLocaleString() }} · {{ item.latest_event.actor }}</p>
            </div>
            <div v-if="openHistory[item.variant_id]" class="inventory-history-panel">
              <p v-if="historyErrors[item.variant_id]" class="admin-feedback">{{ historyErrors[item.variant_id] }}</p>
              <div v-else-if="historyRecords[item.variant_id]?.length" class="inventory-history-list">
                <article
                  v-for="event in historyRecords[item.variant_id]"
                  :key="`${event.event_type}-${event.created_at}-${event.reason}`"
                  class="inventory-history-card"
                >
                  <div class="inventory-history-header">
                    <span class="admin-status-pill admin-status-pill-secondary">{{ event.event_type.replaceAll('_', ' ') }}</span>
                    <strong>{{ new Date(event.created_at).toLocaleString() }}</strong>
                  </div>
                  <p>{{ event.message }}</p>
                  <p>Reason: {{ event.reason }}</p>
                  <p>
                    Stock:
                    {{ event.previous_stock_quantity ?? 'n/a' }}
                    ->
                    {{ event.next_stock_quantity ?? 'n/a' }}
                  </p>
                  <p>
                    Status:
                    {{ event.previous_is_active === null || event.previous_is_active === undefined ? 'n/a' : event.previous_is_active ? 'active' : 'inactive' }}
                    ->
                    {{ event.next_is_active === null || event.next_is_active === undefined ? 'n/a' : event.next_is_active ? 'active' : 'inactive' }}
                  </p>
                  <p>Actor: {{ event.actor }}</p>
                </article>
              </div>
              <p v-else class="route-copy">No adjustment history recorded yet.</p>
            </div>
          </article>
        </div>
      </div>
    </section>
  </AdminShell>
</template>
