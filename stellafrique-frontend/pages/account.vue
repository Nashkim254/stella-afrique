<script setup lang="ts">
const config = useRuntimeConfig()
const { currentCustomer, logoutCustomer, refreshCustomerSession } = useCustomerAuth()
const { formatPrice } = useStorefrontContent()
const { addItem } = useCart()
const profileFeedback = ref('')
const passwordFeedback = ref('')
const orderFeedback = ref('')
const isSavingProfile = ref(false)
const isChangingPassword = ref(false)
const isReordering = ref(false)
const isRefreshingPayment = ref(false)
const isRetryingPayment = ref(false)

await refreshCustomerSession()

const { data: profile } = await useFetch<{
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
}>('/account/profile', {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include',
  server: false,
})

const { data: orders } = await useFetch<Array<{
  order_number: string
  status: string
  payment_status: string
  currency: string
  total_amount: string | number
  item_count: number
  created_at: string
}>>('/account/orders', {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include',
  default: () => [],
  server: false,
})

const selectedOrderNumber = ref('')

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

const { data: selectedOrder } = await useFetch<{
  order_number: string
  status: string
  payment_status: string
  payment_method?: string | null
  payment_reference?: string | null
  paid_at?: string | null
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
  shipping_courier?: string | null
  tracking_number?: string | null
  created_at: string
  items: Array<{
    product_slug?: string | null
    variant_id?: string | null
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
} | null>(
  () => selectedOrderNumber.value ? `/account/orders/${selectedOrderNumber.value}` : null,
  {
    baseURL: config.public.apiBaseUrl,
    credentials: 'include',
    default: () => null,
    server: false,
    watch: [selectedOrderNumber],
  },
)

const profileForm = reactive({
  full_name: '',
  phone: '',
  address_line1: '',
  address_line2: '',
  city: '',
  region: '',
  postal_code: '',
  country: 'Kenya',
})

const passwordForm = reactive({
  current_password: '',
  new_password: '',
  confirm_password: '',
})

watch(
  profile,
  (value) => {
    if (!value) {
      return
    }

    profileForm.full_name = value.full_name ?? ''
    profileForm.phone = value.phone ?? ''
    profileForm.address_line1 = value.address_line1 ?? ''
    profileForm.address_line2 = value.address_line2 ?? ''
    profileForm.city = value.city ?? ''
    profileForm.region = value.region ?? ''
    profileForm.postal_code = value.postal_code ?? ''
    profileForm.country = value.country ?? 'Kenya'
  },
  { immediate: true },
)

const signOut = async () => {
  await logoutCustomer()
  await navigateTo('/')
}

const saveProfile = async () => {
  isSavingProfile.value = true
  profileFeedback.value = ''

  try {
    const updated = await $fetch<{
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
    }>('/account/profile', {
      method: 'PATCH',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        full_name: profileForm.full_name,
        phone: profileForm.phone,
        address_line1: profileForm.address_line1,
        address_line2: profileForm.address_line2,
        city: profileForm.city,
        region: profileForm.region,
        postal_code: profileForm.postal_code,
        country: profileForm.country,
      },
    })

    profile.value = updated
    await refreshCustomerSession(true)
    profileFeedback.value = 'Profile updated.'
  }
  catch (error: any) {
    profileFeedback.value = error?.data?.error ?? 'Unable to save profile.'
  }
  finally {
    isSavingProfile.value = false
  }
}

const changePassword = async () => {
  passwordFeedback.value = ''

  if (!passwordForm.current_password || !passwordForm.new_password) {
    passwordFeedback.value = 'Current and new password are required.'
    return
  }

  if (passwordForm.new_password.length < 8) {
    passwordFeedback.value = 'New password must be at least 8 characters.'
    return
  }

  if (passwordForm.new_password !== passwordForm.confirm_password) {
    passwordFeedback.value = 'Password confirmation does not match.'
    return
  }

  isChangingPassword.value = true

  try {
    await $fetch('/account/password', {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        current_password: passwordForm.current_password,
        new_password: passwordForm.new_password,
      },
    })

    passwordForm.current_password = ''
    passwordForm.new_password = ''
    passwordForm.confirm_password = ''
    passwordFeedback.value = 'Password changed successfully.'
  }
  catch (error: any) {
    passwordFeedback.value = error?.data?.error ?? 'Unable to change password.'
  }
  finally {
    isChangingPassword.value = false
  }
}

const reorderSelectedOrder = async () => {
  if (!selectedOrder.value?.items.length) {
    orderFeedback.value = 'This order has no items to reorder.'
    return
  }

  isReordering.value = true
  orderFeedback.value = ''

  try {
    let addedCount = 0
    let skippedCount = 0

    for (const item of selectedOrder.value.items) {
      if (!item.product_slug) {
        skippedCount += 1
        continue
      }

      addItem({
        lineId: item.variant_id ?? item.product_slug,
        slug: item.product_slug,
        name: item.product_name,
        category: 'Reordered item',
        image: item.image_url || '/images/products/fashion-01.jpg',
        price: String(item.unit_price),
        variantId: item.variant_id ?? undefined,
        variantName: item.variant_name ?? undefined,
        sku: item.sku ?? undefined,
        size: item.size ?? undefined,
        color: item.color ?? undefined,
      }, item.quantity)
      addedCount += 1
    }

    if (!addedCount) {
      orderFeedback.value = 'No active catalog items from this order are available to reorder.'
      return
    }

    orderFeedback.value = skippedCount
      ? `${addedCount} item${addedCount === 1 ? '' : 's'} added to cart. ${skippedCount} line${skippedCount === 1 ? '' : 's'} skipped because the product is no longer available.`
      : `Order items added to cart.`

    await navigateTo('/cart')
  }
  finally {
    isReordering.value = false
  }
}

const refreshSelectedOrderPayment = async () => {
  if (!selectedOrder.value) {
    return
  }

  isRefreshingPayment.value = true
  orderFeedback.value = ''

  try {
    const response = await $fetch<{
      order_number: string
      payment_status: string
      payment_method?: string | null
      payment_reference?: string | null
      provider_status?: string | null
      paid_at?: string | null
    }>(`/payments/velipay/orders/${selectedOrder.value.order_number}/status`, {
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
    })

    selectedOrder.value.payment_status = response.payment_status
    selectedOrder.value.payment_method = response.payment_method ?? selectedOrder.value.payment_method
    selectedOrder.value.payment_reference = response.payment_reference ?? selectedOrder.value.payment_reference
    selectedOrder.value.paid_at = response.paid_at ?? selectedOrder.value.paid_at
    orderFeedback.value = response.payment_status === 'paid'
      ? `Payment confirmed for ${selectedOrder.value.order_number}.`
      : `Payment is still ${response.provider_status || response.payment_status}.`
  }
  catch (error: any) {
    orderFeedback.value = error?.data?.error ?? 'Unable to refresh payment status.'
  }
  finally {
    isRefreshingPayment.value = false
  }
}

const retrySelectedOrderPayment = async () => {
  if (!selectedOrder.value) {
    return
  }

  isRetryingPayment.value = true
  orderFeedback.value = ''

  try {
    const response = await $fetch<{
      provider: string
      method: string
      status: string
      payment_reference?: string | null
      merchant_reference: string
      message: string
    }>(`/payments/velipay/orders/${selectedOrder.value.order_number}/retry`, {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        phone_number: selectedOrder.value.customer_phone || currentCustomer.value?.phone || null,
      },
    })

    selectedOrder.value.payment_method = response.method
    selectedOrder.value.payment_reference = response.payment_reference ?? selectedOrder.value.payment_reference
    orderFeedback.value = response.message
  }
  catch (error: any) {
    orderFeedback.value = error?.data?.error ?? 'Unable to retry payment.'
  }
  finally {
    isRetryingPayment.value = false
  }
}
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container route-hero-inner">
        <div>
          <p class="route-kicker">My Account</p>
          <h1>Welcome back, {{ profile?.full_name || currentCustomer?.full_name || 'Customer' }}.</h1>
          <p class="route-copy">
            Your account is active and signed-in checkout now links new orders directly to this profile.
          </p>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container admin-grid">
        <div class="admin-panel">
          <h2>Profile</h2>
          <p v-if="profileFeedback" class="admin-feedback">{{ profileFeedback }}</p>
          <div class="admin-order-summary-grid">
            <div>
              <span>Name</span>
              <strong>{{ profile?.full_name || currentCustomer?.full_name }}</strong>
            </div>
            <div>
              <span>Email</span>
              <strong>{{ profile?.email || currentCustomer?.email }}</strong>
            </div>
            <div>
              <span>Member Since</span>
              <strong>{{ profile ? new Date(profile.created_at).toLocaleDateString() : 'Recently' }}</strong>
            </div>
          </div>
          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Full Name</span>
              <input v-model="profileForm.full_name" type="text">
            </label>
            <label class="shop-field">
              <span>Email</span>
              <input :value="profile?.email || currentCustomer?.email" type="email" disabled>
            </label>
            <label class="shop-field">
              <span>Phone</span>
              <input v-model="profileForm.phone" type="tel">
            </label>
            <label class="shop-field">
              <span>Country</span>
              <input v-model="profileForm.country" type="text">
            </label>
            <label class="shop-field admin-field-span">
              <span>Address Line 1</span>
              <input v-model="profileForm.address_line1" type="text">
            </label>
            <label class="shop-field admin-field-span">
              <span>Address Line 2</span>
              <input v-model="profileForm.address_line2" type="text">
            </label>
            <label class="shop-field">
              <span>City</span>
              <input v-model="profileForm.city" type="text">
            </label>
            <label class="shop-field">
              <span>Region</span>
              <input v-model="profileForm.region" type="text">
            </label>
            <label class="shop-field">
              <span>Postal Code</span>
              <input v-model="profileForm.postal_code" type="text">
            </label>
          </div>
          <button type="button" class="hero-cta admin-submit" :disabled="isSavingProfile" @click="saveProfile">
            {{ isSavingProfile ? 'Saving...' : 'Save Profile' }}
          </button>
          <button type="button" class="hero-cta admin-submit" @click="signOut">
            Sign Out
          </button>
        </div>
        <div class="admin-panel">
          <h2>Security</h2>
          <p class="route-copy">Change your password without leaving your customer account.</p>
          <p v-if="passwordFeedback" class="admin-feedback">{{ passwordFeedback }}</p>
          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Current Password</span>
              <input v-model="passwordForm.current_password" type="password" autocomplete="current-password">
            </label>
            <label class="shop-field">
              <span>New Password</span>
              <input v-model="passwordForm.new_password" type="password" autocomplete="new-password">
            </label>
            <label class="shop-field">
              <span>Confirm New Password</span>
              <input v-model="passwordForm.confirm_password" type="password" autocomplete="new-password">
            </label>
          </div>
          <button type="button" class="hero-cta admin-submit" :disabled="isChangingPassword" @click="changePassword">
            {{ isChangingPassword ? 'Updating Password...' : 'Change Password' }}
          </button>
        </div>
        <div class="admin-panel">
          <h2>Order History</h2>
          <div v-if="orders?.length" class="admin-table">
            <article
              v-for="order in orders"
              :key="order.order_number"
              class="admin-row admin-order-row"
              :class="{ 'is-active': selectedOrderNumber === order.order_number }"
              @click="selectedOrderNumber = order.order_number"
            >
              <div>
                <strong>{{ order.order_number }}</strong>
                <p>{{ new Date(order.created_at).toLocaleString() }} · {{ order.item_count }} items</p>
              </div>
              <div class="admin-order-meta">
                <span class="admin-status-pill">{{ order.status }}</span>
                <span class="admin-status-pill admin-status-pill-secondary">{{ order.payment_status }}</span>
                <strong>{{ order.currency }} {{ order.total_amount }}</strong>
              </div>
            </article>
          </div>
          <p v-else class="route-copy">No orders linked to this account yet.</p>
        </div>
        <div v-if="selectedOrder" class="admin-panel">
          <h2>Order Detail</h2>
          <p v-if="orderFeedback" class="admin-feedback">{{ orderFeedback }}</p>
          <div class="admin-order-detail">
            <div class="admin-order-actions">
              <button type="button" class="hero-cta admin-submit" :disabled="isReordering" @click="reorderSelectedOrder">
                {{ isReordering ? 'Rebuilding Cart...' : 'Reorder Items' }}
              </button>
              <button
                v-if="selectedOrder.payment_method === 'velipay_stk_push' && selectedOrder.payment_status !== 'paid' && selectedOrder.status !== 'cancelled'"
                type="button"
                class="secondary-link"
                :disabled="isRetryingPayment"
                @click="retrySelectedOrderPayment"
              >
                {{ isRetryingPayment ? 'Retrying Payment...' : 'Retry Payment' }}
              </button>
              <button
                v-if="selectedOrder.payment_method === 'velipay_stk_push' && selectedOrder.payment_status !== 'paid'"
                type="button"
                class="secondary-link"
                :disabled="isRefreshingPayment"
                @click="refreshSelectedOrderPayment"
              >
                {{ isRefreshingPayment ? 'Refreshing...' : 'Refresh Payment Status' }}
              </button>
            </div>
            <div class="admin-order-summary-grid">
              <div>
                <span>Order Number</span>
                <strong>{{ selectedOrder.order_number }}</strong>
              </div>
              <div>
                <span>Created</span>
                <strong>{{ new Date(selectedOrder.created_at).toLocaleString() }}</strong>
              </div>
              <div>
                <span>Fulfilment</span>
                <strong>{{ selectedOrder.status }}</strong>
              </div>
              <div>
                <span>Payment</span>
                <strong>{{ selectedOrder.payment_status }}</strong>
              </div>
              <div v-if="selectedOrder.payment_reference">
                <span>Reference</span>
                <strong>{{ selectedOrder.payment_reference }}</strong>
              </div>
              <div v-if="selectedOrder.paid_at">
                <span>Paid At</span>
                <strong>{{ new Date(selectedOrder.paid_at).toLocaleString() }}</strong>
              </div>
              <div>
                <span>Courier</span>
                <strong>{{ selectedOrder.shipping_courier || 'Not assigned' }}</strong>
              </div>
              <div>
                <span>Tracking</span>
                <strong>{{ selectedOrder.tracking_number || 'Not assigned' }}</strong>
              </div>
            </div>

            <div class="admin-order-summary-grid">
              <div>
                <span>Shipping Address</span>
                <strong>
                  {{ selectedOrder.shipping_address_line1 }}
                  <template v-if="selectedOrder.shipping_address_line2">, {{ selectedOrder.shipping_address_line2 }}</template>
                  , {{ selectedOrder.shipping_city }}
                  <template v-if="selectedOrder.shipping_region">, {{ selectedOrder.shipping_region }}</template>
                  <template v-if="selectedOrder.shipping_postal_code">, {{ selectedOrder.shipping_postal_code }}</template>
                  , {{ selectedOrder.shipping_country }}
                </strong>
              </div>
              <div>
                <span>Notes</span>
                <strong>{{ selectedOrder.notes || 'No delivery notes' }}</strong>
              </div>
            </div>

            <div class="admin-order-lines">
              <article v-for="item in selectedOrder.items" :key="`${selectedOrder.order_number}-${item.sku || item.product_name}`" class="admin-order-line">
                <div>
                  <strong>{{ item.product_name }}</strong>
                  <p v-if="item.variant_name">{{ item.variant_name }}</p>
                  <p v-if="item.size || item.color">
                    <template v-if="item.size">Size: {{ item.size }}</template>
                    <template v-if="item.size && item.color"> · </template>
                    <template v-if="item.color">Colour: {{ item.color }}</template>
                  </p>
                  <p>Qty: {{ item.quantity }}</p>
                </div>
                <strong>{{ formatPrice(String(item.line_total)) }}</strong>
              </article>
            </div>

            <div class="admin-order-summary-grid">
              <div>
                <span>Subtotal</span>
                <strong>{{ formatPrice(String(selectedOrder.subtotal_amount)) }}</strong>
              </div>
              <div>
                <span>Total</span>
                <strong>{{ formatPrice(String(selectedOrder.total_amount)) }}</strong>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
