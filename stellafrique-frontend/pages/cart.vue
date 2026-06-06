<script setup lang="ts">
const { formatPrice } = useStorefrontContent()
const { items, removeItem, updateQuantity, clear, subtotal } = useCart()
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container route-hero-inner">
        <div>
          <p class="route-kicker">Cart</p>
          <h1>Your selected wardrobe pieces.</h1>
          <p class="route-copy">
            Review quantities, remove items, and keep building the edit before checkout is connected.
          </p>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container">
        <div v-if="items.length" class="cart-layout">
          <div class="cart-items">
            <article v-for="item in items" :key="item.lineId" class="cart-item-card">
              <div class="cart-item-media">
                <img :src="item.image" :alt="item.name">
              </div>
              <div class="cart-item-copy">
                <span>{{ item.category }}</span>
                <h3>{{ item.name }}</h3>
                <p v-if="item.variantName">{{ item.variantName }}</p>
                <p v-if="item.size || item.color">
                  <template v-if="item.size">Size: {{ item.size }}</template>
                  <template v-if="item.size && item.color"> · </template>
                  <template v-if="item.color">Colour: {{ item.color }}</template>
                </p>
                <p v-if="item.sku">SKU: {{ item.sku }}</p>
                <div class="cart-quantity-control">
                  <button type="button" class="secondary-link cart-quantity-button" @click="updateQuantity(item.lineId, item.quantity - 1)">
                    -
                  </button>
                  <strong>{{ item.quantity }}</strong>
                  <button type="button" class="secondary-link cart-quantity-button" @click="updateQuantity(item.lineId, item.quantity + 1)">
                    +
                  </button>
                </div>
              </div>
              <div class="cart-item-actions">
                <strong>{{ formatPrice(item.price) }}</strong>
                <button type="button" class="secondary-link" @click="removeItem(item.lineId)">
                  Remove
                </button>
              </div>
            </article>
          </div>

          <aside class="cart-summary">
            <h3>Order Summary</h3>
            <p>Subtotal</p>
            <strong>{{ formatPrice(String(subtotal)) }}</strong>
            <NuxtLink to="/checkout" class="hero-cta cart-summary-button">Continue To Checkout</NuxtLink>
            <button type="button" class="secondary-link cart-summary-button" @click="clear">
              Clear Cart
            </button>
          </aside>
        </div>

        <div v-else class="empty-state">
          <h2>Your cart is empty.</h2>
          <p>Start with the latest arrivals or browse a collection to add your first piece.</p>
          <NuxtLink to="/shop" class="hero-cta">Continue Shopping</NuxtLink>
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
