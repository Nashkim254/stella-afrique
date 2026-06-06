<script setup lang="ts">
const config = useRuntimeConfig()
const router = useRouter()
const { formatPrice } = useStorefrontContent()
const { items, subtotal, clear } = useCart()
const { currentCustomer, refreshCustomerSession } = useCustomerAuth()

type OrderResponse = {
  order_number: string
  status: string
  payment_status: string
  currency: string
  subtotal_amount: string | number
  total_amount: string | number
  customer_name: string
  customer_email: string
  payment?: {
    provider: string
    method: string
    status: string
    payment_reference?: string | null
    merchant_reference: string
    message: string
  } | null
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
}

const checkoutForm = reactive({
  customer_name: '',
  customer_email: '',
  customer_phone: '',
  shipping_address_line1: '',
  shipping_address_line2: '',
  shipping_city: '',
  shipping_region: '',
  shipping_postal_code: '',
  shipping_country: '',
  notes: '',
})

const isSubmitting = ref(false)
const isRefreshingPayment = ref(false)
const isRetryingPayment = ref(false)
const feedback = ref('')
const orderResult = ref<OrderResponse | null>(null)
const syncProfileAfterOrder = ref(true)
const selectedPaymentMethod = ref<'velipay_stk_push' | 'manual'>('velipay_stk_push')

await refreshCustomerSession()

watchEffect(() => {
  if (currentCustomer.value) {
    if (!checkoutForm.customer_name) {
      checkoutForm.customer_name = currentCustomer.value.full_name
    }
    if (!checkoutForm.customer_email) {
      checkoutForm.customer_email = currentCustomer.value.email
    }
    if (!checkoutForm.customer_phone) {
      checkoutForm.customer_phone = currentCustomer.value.phone ?? ''
    }
    if (!checkoutForm.shipping_address_line1) {
      checkoutForm.shipping_address_line1 = currentCustomer.value.address_line1 ?? ''
    }
    if (!checkoutForm.shipping_address_line2) {
      checkoutForm.shipping_address_line2 = currentCustomer.value.address_line2 ?? ''
    }
    if (!checkoutForm.shipping_city) {
      checkoutForm.shipping_city = currentCustomer.value.city ?? ''
    }
    if (!checkoutForm.shipping_region) {
      checkoutForm.shipping_region = currentCustomer.value.region ?? ''
    }
    if (!checkoutForm.shipping_postal_code) {
      checkoutForm.shipping_postal_code = currentCustomer.value.postal_code ?? ''
    }
    if (!checkoutForm.shipping_country) {
      checkoutForm.shipping_country = currentCustomer.value.country ?? 'Kenya'
    }
  }
  else if (!checkoutForm.shipping_country) {
    checkoutForm.shipping_country = 'Kenya'
  }
})

if (import.meta.client && !items.value.length) {
  router.replace('/cart')
}

const normalizeProfileValue = (value?: string | null) => value?.trim() ?? ''

const shouldSyncProfile = computed(() => {
  const customer = currentCustomer.value
  if (!customer || !syncProfileAfterOrder.value) {
    return false
  }

  return (
    normalizeProfileValue(customer.full_name) !== normalizeProfileValue(checkoutForm.customer_name)
    || normalizeProfileValue(customer.phone) !== normalizeProfileValue(checkoutForm.customer_phone)
    || normalizeProfileValue(customer.address_line1) !== normalizeProfileValue(checkoutForm.shipping_address_line1)
    || normalizeProfileValue(customer.address_line2) !== normalizeProfileValue(checkoutForm.shipping_address_line2)
    || normalizeProfileValue(customer.city) !== normalizeProfileValue(checkoutForm.shipping_city)
    || normalizeProfileValue(customer.region) !== normalizeProfileValue(checkoutForm.shipping_region)
    || normalizeProfileValue(customer.postal_code) !== normalizeProfileValue(checkoutForm.shipping_postal_code)
    || normalizeProfileValue(customer.country) !== normalizeProfileValue(checkoutForm.shipping_country)
  )
})

const syncCustomerProfileFromCheckout = async () => {
  if (!shouldSyncProfile.value) {
    return false
  }

  await $fetch('/account/profile', {
    method: 'PATCH',
    baseURL: config.public.apiBaseUrl,
    credentials: 'include',
    body: {
      full_name: checkoutForm.customer_name,
      phone: checkoutForm.customer_phone || null,
      address_line1: checkoutForm.shipping_address_line1 || null,
      address_line2: checkoutForm.shipping_address_line2 || null,
      city: checkoutForm.shipping_city || null,
      region: checkoutForm.shipping_region || null,
      postal_code: checkoutForm.shipping_postal_code || null,
      country: checkoutForm.shipping_country || null,
    },
  })

  await refreshCustomerSession(true)
  return true
}

const submitOrder = async () => {
  if (!items.value.length) {
    feedback.value = 'Your cart is empty.'
    return
  }

  isSubmitting.value = true
  feedback.value = ''

  try {
    const needsProfileSync = shouldSyncProfile.value
    const response = await $fetch<OrderResponse>('/orders', {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        ...checkoutForm,
        shipping_address_line2: checkoutForm.shipping_address_line2 || null,
        shipping_region: checkoutForm.shipping_region || null,
        shipping_postal_code: checkoutForm.shipping_postal_code || null,
        notes: checkoutForm.notes || null,
        payment_method: selectedPaymentMethod.value === 'velipay_stk_push' ? 'velipay_stk_push' : null,
        payment_phone_number: selectedPaymentMethod.value === 'velipay_stk_push'
          ? (checkoutForm.customer_phone || null)
          : null,
        items: items.value.map((item) => ({
          slug: item.slug,
          variant_id: item.variantId ?? null,
          quantity: item.quantity,
        })),
      },
    })

    let profileSynced = false
    if (needsProfileSync) {
      try {
        profileSynced = await syncCustomerProfileFromCheckout()
      }
      catch {
        profileSynced = false
      }
    }
    orderResult.value = response
    clear()
    feedback.value = profileSynced
      ? `Order ${response.order_number} created and your account profile was updated.`
      : needsProfileSync
        ? `Order ${response.order_number} created. Payment was started, but your saved profile could not be updated from checkout.`
      : `Order ${response.order_number} created.`
  }
  catch (error: any) {
    feedback.value = error?.data?.error ?? 'Failed to place order.'
  }
  finally {
    isSubmitting.value = false
  }
}

const refreshPaymentStatus = async () => {
  if (!orderResult.value) {
    return
  }

  isRefreshingPayment.value = true
  feedback.value = ''

  try {
    const response = await $fetch<{
      order_number: string
      payment_status: string
      payment_method?: string | null
      payment_reference?: string | null
      provider_status?: string | null
      paid_at?: string | null
    }>(`/payments/velipay/orders/${orderResult.value.order_number}/status`, {
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      query: {
        customer_email: orderResult.value.customer_email,
      },
    })

    orderResult.value.payment_status = response.payment_status
    if (orderResult.value.payment) {
      orderResult.value.payment.status = response.provider_status || response.payment_status
      orderResult.value.payment.payment_reference = response.payment_reference ?? orderResult.value.payment.payment_reference
    }
    feedback.value = response.payment_status === 'paid'
      ? `Payment confirmed for ${orderResult.value.order_number}.`
      : `Payment is still ${response.provider_status || response.payment_status}.`
  }
  catch (error: any) {
    feedback.value = error?.data?.error ?? 'Unable to refresh payment status.'
  }
  finally {
    isRefreshingPayment.value = false
  }
}

const retryPayment = async () => {
  if (!orderResult.value) {
    return
  }

  isRetryingPayment.value = true
  feedback.value = ''

  try {
    const result = await $fetch<{
      provider: string
      method: string
      status: string
      payment_reference?: string | null
      merchant_reference: string
      message: string
    }>(`/payments/velipay/orders/${orderResult.value.order_number}/retry`, {
      method: 'POST',
      baseURL: config.public.apiBaseUrl,
      credentials: 'include',
      body: {
        phone_number: checkoutForm.customer_phone || null,
        customer_email: orderResult.value.customer_email,
      },
    })

    if (orderResult.value.payment) {
      orderResult.value.payment.status = result.status
      orderResult.value.payment.payment_reference = result.payment_reference ?? orderResult.value.payment.payment_reference
      orderResult.value.payment.message = result.message
    }
    feedback.value = result.message
  }
  catch (error: any) {
    feedback.value = error?.data?.error ?? 'Unable to retry payment.'
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
      <div class="container route-hero-inner route-hero-split">
        <div>
          <p class="route-kicker">Checkout</p>
          <h1>Finish the order and confirm delivery details.</h1>
          <p class="route-copy">
            This creates a real order record in the Rust backend and can now trigger a Velipay STK push for immediate customer payment.
          </p>
          <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>
        </div>
        <div class="route-hero-media">
          <img src="/images/products/fashion-15.jpg" alt="Checkout editorial">
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container checkout-layout">
        <div v-if="!orderResult" class="admin-panel">
          <h2>Customer & Shipping</h2>
          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Full Name</span>
              <input v-model="checkoutForm.customer_name" type="text" placeholder="Mary Atieno">
            </label>
            <label class="shop-field">
              <span>Email</span>
              <input v-model="checkoutForm.customer_email" type="email" placeholder="mary@example.com">
            </label>
            <label class="shop-field">
              <span>Phone</span>
              <input v-model="checkoutForm.customer_phone" type="tel" placeholder="+254 700 000 000">
            </label>
            <label class="shop-field">
              <span>Country</span>
              <input v-model="checkoutForm.shipping_country" type="text" placeholder="Kenya">
            </label>
            <label class="shop-field admin-field-span">
              <span>Address Line 1</span>
              <input v-model="checkoutForm.shipping_address_line1" type="text" placeholder="Apartment, street, estate">
            </label>
            <label class="shop-field admin-field-span">
              <span>Address Line 2</span>
              <input v-model="checkoutForm.shipping_address_line2" type="text" placeholder="Optional extra address details">
            </label>
            <label class="shop-field">
              <span>City</span>
              <input v-model="checkoutForm.shipping_city" type="text" placeholder="Nairobi">
            </label>
            <label class="shop-field">
              <span>Region</span>
              <input v-model="checkoutForm.shipping_region" type="text" placeholder="Nairobi County">
            </label>
            <label class="shop-field">
              <span>Postal Code</span>
              <input v-model="checkoutForm.shipping_postal_code" type="text" placeholder="00100">
            </label>
            <label class="shop-field admin-field-span">
              <span>Notes</span>
              <textarea v-model="checkoutForm.notes" rows="4" placeholder="Delivery instructions, gate code, or gift note." />
            </label>
          </div>
          <div class="checkout-payment-panel">
            <h3>Payment</h3>
            <label class="shop-field">
              <span>Payment Method</span>
              <select v-model="selectedPaymentMethod">
                <option value="velipay_stk_push">Velipay STK Push</option>
                <option value="manual">Manual Follow-up</option>
              </select>
            </label>
            <p class="checkout-sync-note">
              <template v-if="selectedPaymentMethod === 'velipay_stk_push'">
                The backend will create the order and immediately ask Velipay to push an STK prompt to the phone number above.
              </template>
              <template v-else>
                The order will be created without starting a payment request.
              </template>
            </p>
          </div>
          <label v-if="currentCustomer" class="checkout-sync-toggle">
            <input v-model="syncProfileAfterOrder" type="checkbox">
            <span>Save these delivery details to my account for the next order.</span>
          </label>
          <p v-if="currentCustomer && shouldSyncProfile" class="checkout-sync-note">
            Your account profile will be updated with the delivery details above after this order is placed.
          </p>
          <button type="button" class="hero-cta admin-submit" :disabled="isSubmitting || !items.length" @click="submitOrder">
            {{ isSubmitting ? 'Placing Order...' : 'Place Order' }}
          </button>
        </div>

        <div v-else class="admin-panel checkout-success">
          <p class="route-kicker">Order Confirmed</p>
          <h2>{{ orderResult.order_number }}</h2>
          <p class="route-copy">
            A live order has been created for {{ orderResult.customer_name }}. Payment can settle automatically through Velipay webhook delivery, and you can still refresh or retry if the provider response is delayed.
          </p>
          <div class="checkout-success-grid">
            <div>
              <span>Status</span>
              <strong>{{ orderResult.status }}</strong>
            </div>
            <div>
              <span>Payment</span>
              <strong>{{ orderResult.payment_status }}</strong>
            </div>
            <div>
              <span>Total</span>
              <strong>{{ formatPrice(String(orderResult.total_amount)) }}</strong>
            </div>
          </div>
          <div v-if="orderResult.payment" class="checkout-payment-result">
            <p class="route-copy">{{ orderResult.payment.message }}</p>
            <p v-if="orderResult.payment.payment_reference" class="checkout-sync-note">
              Payment reference: <strong>{{ orderResult.payment.payment_reference }}</strong>
            </p>
            <button
              v-if="orderResult.payment.provider === 'velipay'"
              type="button"
              class="secondary-link"
              :disabled="isRefreshingPayment"
              @click="refreshPaymentStatus"
            >
              {{ isRefreshingPayment ? 'Refreshing Payment...' : 'Refresh Payment Status' }}
            </button>
            <button
              v-if="orderResult.payment.provider === 'velipay' && orderResult.payment_status !== 'paid'"
              type="button"
              class="secondary-link"
              :disabled="isRetryingPayment"
              @click="retryPayment"
            >
              {{ isRetryingPayment ? 'Retrying Payment...' : 'Retry Payment' }}
            </button>
          </div>
          <div class="admin-order-actions">
            <NuxtLink v-if="currentCustomer" to="/account" class="secondary-link">View My Orders</NuxtLink>
            <NuxtLink to="/shop" class="hero-cta">Continue Shopping</NuxtLink>
          </div>
        </div>

        <aside class="cart-summary checkout-summary">
          <h3>Order Summary</h3>
          <div v-if="items.length" class="checkout-line-list">
            <article v-for="item in items" :key="item.lineId" class="checkout-line-item">
              <img :src="item.image" :alt="item.name">
              <div>
                <strong>{{ item.name }}</strong>
                <p v-if="item.variantName">{{ item.variantName }}</p>
                <p v-if="item.size || item.color">
                  <template v-if="item.size">Size: {{ item.size }}</template>
                  <template v-if="item.size && item.color"> · </template>
                  <template v-if="item.color">Colour: {{ item.color }}</template>
                </p>
                <p>Qty: {{ item.quantity }}</p>
              </div>
              <strong>{{ formatPrice(item.price) }}</strong>
            </article>
          </div>
          <div v-else class="empty-state compact-empty">
            <p>Your cart is empty.</p>
            <NuxtLink to="/shop" class="secondary-link">Back To Shop</NuxtLink>
          </div>
          <p>Subtotal</p>
          <strong>{{ formatPrice(String(subtotal)) }}</strong>
        </aside>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
