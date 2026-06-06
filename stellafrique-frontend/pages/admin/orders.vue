<script setup lang="ts">
const config = useRuntimeConfig()
const { formatPrice } = useStorefrontContent()

type AdminOrderListItem = {
  order_number: string
  status: string
  payment_status: string
  currency: string
  total_amount: string | number
  customer_name: string
  customer_email: string
  item_count: number
  created_at: string
}

type AdminOrderDetail = {
  order_number: string
  status: string
  payment_status: string
  currency: string
  subtotal_amount: string | number
  total_amount: string | number
  customer_name: string
  customer_email: string
  customer_phone: string
  shipping_address_line1: string
  shipping_address_line2?: string | null
  shipping_city: string
  shipping_region?: string | null
  shipping_postal_code?: string | null
  shipping_country: string
  notes?: string | null
  payment_method?: string | null
  payment_reference?: string | null
  provider_status?: string | null
  shipping_courier?: string | null
  tracking_number?: string | null
  paid_at?: string | null
  fulfilled_at?: string | null
  created_at: string
  items: Array<{
    product_name: string
    variant_name?: string | null
    sku?: string | null
    size?: string | null
    color?: string | null
    quantity: number
    unit_price: string | number
    line_total: string | number
    image_url?: string | null
  }>
  events: Array<{
    event_type: string
    actor: string
    message: string
    details?: string | null
    created_at: string
  }>
}

const feedback = ref('')
const search = ref('')
const listStatus = ref('')
const listPaymentStatus = ref('')
const selectedOrderNumber = ref('')
const selectedStatus = ref('pending')
const selectedPaymentStatus = ref('unpaid')
const paymentMethod = ref('')
const paymentReference = ref('')
const paymentProviderStatus = ref('')
const shippingCourier = ref('')
const trackingNumber = ref('')
const isUpdating = ref(false)
const isRefreshingPayment = ref(false)
const isRetryingPayment = ref(false)
const adminFetchOptions = {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include' as const,
}

const extractErrorMessage = (error: any, fallback: string) =>
  error?.data?.error
  ?? error?.data?.message
  ?? error?.statusMessage
  ?? error?.message
  ?? fallback

const orderListQuery = computed(() => ({
  status: listStatus.value || undefined,
  payment_status: listPaymentStatus.value || undefined,
  search: search.value.trim() || undefined,
}))

const { data: orders, refresh } = await useFetch<AdminOrderListItem[]>('/admin/orders', {
  ...adminFetchOptions,
  query: orderListQuery,
  default: () => [],
  server: false,
  watch: [orderListQuery],
})

watch(
  orders,
  (value) => {
    if (!value?.length) {
      selectedOrderNumber.value = ''
      return
    }

    if (!value.some((order) => order.order_number === selectedOrderNumber.value)) {
      selectedOrderNumber.value = value[0].order_number
    }
  },
  { immediate: true },
)

const {
  data: orderDetail,
  refresh: refreshOrderDetail,
} = await useFetch<AdminOrderDetail | null>(
  () => selectedOrderNumber.value ? `/admin/orders/${selectedOrderNumber.value}` : null,
  {
    ...adminFetchOptions,
    default: () => null,
    server: false,
    watch: [selectedOrderNumber],
  },
)

watch(
  orderDetail,
  (value) => {
    if (value) {
      selectedStatus.value = value.status
      selectedPaymentStatus.value = value.payment_status
      paymentMethod.value = value.payment_method ?? ''
      paymentReference.value = value.payment_reference ?? ''
      paymentProviderStatus.value = value.provider_status ?? ''
      shippingCourier.value = value.shipping_courier ?? ''
      trackingNumber.value = value.tracking_number ?? ''
    }
  },
  { immediate: true },
)

const updateStatus = async () => {
  if (!selectedOrderNumber.value) {
    return
  }

  isUpdating.value = true
  feedback.value = ''

  try {
    await $fetch(`/admin/orders/${selectedOrderNumber.value}`, {
      method: 'PATCH',
      ...adminFetchOptions,
      body: {
        status: selectedStatus.value,
        payment_status: selectedPaymentStatus.value,
        payment_method: paymentMethod.value,
        payment_reference: paymentReference.value,
        shipping_courier: shippingCourier.value,
        tracking_number: trackingNumber.value,
      },
    })

    feedback.value = `Order ${selectedOrderNumber.value} updated.`
    await Promise.all([refresh(), refreshOrderDetail()])
  }
  finally {
    isUpdating.value = false
  }
}

const refreshPaymentStatus = async () => {
  if (!selectedOrderNumber.value) {
    return
  }

  isRefreshingPayment.value = true
  feedback.value = ''

  try {
    const status = await $fetch<{
      order_number: string
      payment_status: string
      payment_method?: string | null
      payment_reference?: string | null
      provider_status?: string | null
      paid_at?: string | null
    }>(`/admin/orders/${selectedOrderNumber.value}/payment-status`, {
      ...adminFetchOptions,
    })

    selectedPaymentStatus.value = status.payment_status
    paymentMethod.value = status.payment_method ?? paymentMethod.value
    paymentReference.value = status.payment_reference ?? paymentReference.value
    paymentProviderStatus.value = status.provider_status ?? ''
    feedback.value = `Payment status refreshed for ${status.order_number}.`
    await Promise.all([refresh(), refreshOrderDetail()])
  }
  catch (error: any) {
    feedback.value = extractErrorMessage(error, 'Unable to refresh payment status.')
  }
  finally {
    isRefreshingPayment.value = false
  }
}

const retryPayment = async () => {
  if (!selectedOrderNumber.value || !orderDetail.value) {
    return
  }

  isRetryingPayment.value = true
  feedback.value = ''

  try {
    const result = await $fetch<{
      status: string
      payment_reference?: string | null
      message: string
    }>(`/admin/orders/${selectedOrderNumber.value}/retry-payment`, {
      method: 'POST',
      ...adminFetchOptions,
      body: {
        phone_number: orderDetail.value.customer_phone,
      },
    })

    paymentMethod.value = 'velipay_stk_push'
    if (result.payment_reference) {
      paymentReference.value = result.payment_reference
    }
    paymentProviderStatus.value = result.status
    feedback.value = result.message
    await Promise.all([refresh(), refreshOrderDetail()])
  }
  catch (error: any) {
    feedback.value = extractErrorMessage(error, 'Unable to retry payment.')
  }
  finally {
    isRetryingPayment.value = false
  }
}

</script>

<template>
  <AdminShell
    title="Order operations for Stellafrique."
    description="Review incoming orders, inspect customer details, and move each order through payment and fulfilment states."
  >
    <section class="admin-content-section">
      <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>
      <div class="admin-grid">
        <div class="admin-panel">
          <div class="admin-section-heading">
            <div>
              <h2>Orders</h2>
              <p>{{ orders.length }} matching orders</p>
            </div>
            <NuxtLink to="/admin/catalog" class="secondary-link">Back To Catalog</NuxtLink>
          </div>
          <div class="admin-orders-filters">
            <label class="shop-field inventory-search">
              <span>Search</span>
              <input
                v-model="search"
                type="text"
                placeholder="Order number, customer name, or email"
              >
            </label>
            <label class="shop-field">
              <span>Fulfilment</span>
              <select v-model="listStatus">
                <option value="">All statuses</option>
                <option value="pending">Pending</option>
                <option value="paid">Paid</option>
                <option value="fulfilled">Fulfilled</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </label>
            <label class="shop-field">
              <span>Payment</span>
              <select v-model="listPaymentStatus">
                <option value="">All payment states</option>
                <option value="unpaid">Unpaid</option>
                <option value="paid">Paid</option>
                <option value="refunded">Refunded</option>
              </select>
            </label>
          </div>
          <div class="admin-table">
            <article
              v-for="order in orders"
              :key="order.order_number"
              class="admin-row admin-order-row"
              :class="{ 'is-active': selectedOrderNumber === order.order_number }"
              @click="selectedOrderNumber = order.order_number"
            >
              <div>
                <strong>{{ order.order_number }}</strong>
                <p>{{ order.customer_name }} · {{ order.customer_email }}</p>
              </div>
              <div class="admin-order-meta">
                <span class="admin-status-pill">{{ order.status }}</span>
                <span class="admin-status-pill admin-status-pill-secondary">{{ order.payment_status }}</span>
                <strong>{{ formatPrice(String(order.total_amount)) }}</strong>
              </div>
            </article>
          </div>
        </div>

        <div class="admin-panel">
          <div v-if="orderDetail" class="admin-order-detail">
            <div class="admin-section-heading">
              <div>
                <h2>{{ orderDetail.order_number }}</h2>
                <p>{{ orderDetail.customer_name }} · {{ orderDetail.customer_email }}</p>
              </div>
              <div class="admin-order-status-control">
                <label class="shop-field">
                  <span>Fulfilment</span>
                  <select v-model="selectedStatus">
                    <option value="pending">Pending</option>
                    <option value="paid">Paid</option>
                    <option value="fulfilled">Fulfilled</option>
                    <option value="cancelled">Cancelled</option>
                  </select>
                </label>
                <label class="shop-field">
                  <span>Payment</span>
                  <select v-model="selectedPaymentStatus">
                    <option value="unpaid">Unpaid</option>
                    <option value="paid">Paid</option>
                    <option value="refunded">Refunded</option>
                  </select>
                </label>
                <label class="shop-field">
                  <span>Method</span>
                  <input v-model="paymentMethod" type="text" placeholder="M-Pesa, Card, Bank transfer">
                </label>
                <label class="shop-field">
                  <span>Reference</span>
                  <input v-model="paymentReference" type="text" placeholder="Transaction reference">
                </label>
                <label class="shop-field">
                  <span>Courier</span>
                  <input v-model="shippingCourier" type="text" placeholder="Sendy, Fargo, G4S">
                </label>
                <label class="shop-field">
                  <span>Tracking</span>
                  <input v-model="trackingNumber" type="text" placeholder="Tracking number">
                </label>
                <button type="button" class="hero-cta admin-submit" :disabled="isUpdating" @click="updateStatus">
                  {{ isUpdating ? 'Updating...' : 'Update Status' }}
                </button>
              </div>
            </div>

            <div class="admin-order-summary-grid">
              <div>
                <span>Total</span>
                <strong>{{ formatPrice(String(orderDetail.total_amount)) }}</strong>
              </div>
              <div>
                <span>Payment</span>
                <strong>{{ orderDetail.payment_status }}</strong>
              </div>
              <div>
                <span>Paid At</span>
                <strong>{{ orderDetail.paid_at ? new Date(orderDetail.paid_at).toLocaleString() : 'Not paid yet' }}</strong>
              </div>
              <div>
                <span>Provider Status</span>
                <strong>{{ paymentProviderStatus || 'Not checked' }}</strong>
              </div>
              <div>
                <span>Fulfilled At</span>
                <strong>{{ orderDetail.fulfilled_at ? new Date(orderDetail.fulfilled_at).toLocaleString() : 'Not fulfilled yet' }}</strong>
              </div>
              <div>
                <span>Phone</span>
                <strong>{{ orderDetail.customer_phone }}</strong>
              </div>
              <div>
                <span>Ship To</span>
                <strong>
                  {{ orderDetail.shipping_address_line1 }},
                  {{ orderDetail.shipping_city }},
                  {{ orderDetail.shipping_country }}
                </strong>
              </div>
              <div>
                <span>Placed</span>
                <strong>{{ new Date(orderDetail.created_at).toLocaleString() }}</strong>
              </div>
              <div>
                <span>Courier</span>
                <strong>{{ orderDetail.shipping_courier || 'Not set' }}</strong>
              </div>
              <div>
                <span>Tracking</span>
                <strong>{{ orderDetail.tracking_number || 'Not set' }}</strong>
              </div>
            </div>

            <div v-if="orderDetail.notes" class="product-specs">
              <div>
                <span>Notes</span>
                <strong>{{ orderDetail.notes }}</strong>
              </div>
            </div>

            <div v-if="orderDetail.payment_method || orderDetail.payment_reference" class="product-specs">
              <div v-if="orderDetail.payment_method">
                <span>Payment Method</span>
                <strong>{{ orderDetail.payment_method }}</strong>
              </div>
              <div v-if="orderDetail.payment_reference">
                <span>Payment Reference</span>
                <strong>{{ orderDetail.payment_reference }}</strong>
              </div>
            </div>

            <div class="admin-order-actions">
              <button type="button" class="secondary-link" :disabled="isRefreshingPayment" @click="refreshPaymentStatus">
                {{ isRefreshingPayment ? 'Refreshing Payment...' : 'Refresh Payment Status' }}
              </button>
              <NuxtLink
                v-if="orderDetail.payment_status === 'paid'"
                to="/admin/withdrawals"
                class="secondary-link"
              >
                Open Withdrawals
              </NuxtLink>
              <button
                v-if="orderDetail.payment_status !== 'paid' && orderDetail.status !== 'cancelled'"
                type="button"
                class="hero-cta admin-submit"
                :disabled="isRetryingPayment"
                @click="retryPayment"
              >
                {{ isRetryingPayment ? 'Retrying STK Push...' : 'Retry Payment' }}
              </button>
            </div>

            <div class="admin-order-lines">
              <article v-for="item in orderDetail.items" :key="`${item.product_name}-${item.sku}-${item.quantity}`" class="admin-order-line">
                <img v-if="item.image_url" :src="item.image_url" :alt="item.product_name">
                <div>
                  <strong>{{ item.product_name }}</strong>
                  <p v-if="item.variant_name">{{ item.variant_name }}</p>
                  <p v-if="item.size || item.color">
                    <template v-if="item.size">Size: {{ item.size }}</template>
                    <template v-if="item.size && item.color"> · </template>
                    <template v-if="item.color">Colour: {{ item.color }}</template>
                  </p>
                  <p v-if="item.sku">SKU: {{ item.sku }}</p>
                  <p>Qty: {{ item.quantity }}</p>
                </div>
                <strong>{{ formatPrice(String(item.line_total)) }}</strong>
              </article>
            </div>

            <div class="admin-order-events">
              <div class="admin-section-heading">
                <div>
                  <h2>Timeline</h2>
                  <p>Inventory, status, and email events for this order.</p>
                </div>
              </div>

              <div class="admin-event-list">
                <article
                  v-for="event in orderDetail.events"
                  :key="`${event.event_type}-${event.created_at}-${event.message}`"
                  class="admin-event-card"
                >
                  <div class="admin-event-meta">
                    <span class="admin-status-pill">{{ event.event_type.replaceAll('_', ' ') }}</span>
                    <strong>{{ new Date(event.created_at).toLocaleString() }}</strong>
                  </div>
                  <p class="admin-event-message">{{ event.message }}</p>
                  <p class="admin-event-actor">Actor: {{ event.actor }}</p>
                  <p v-if="event.details" class="admin-event-details">{{ event.details }}</p>
                </article>
              </div>
            </div>
          </div>

          <div v-else class="empty-state compact-empty">
            <h2>No order selected.</h2>
            <p>Create a checkout order first, then manage it here.</p>
          </div>
        </div>
      </div>
    </section>
  </AdminShell>
</template>
