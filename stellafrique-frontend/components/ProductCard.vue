<script setup lang="ts">
defineProps<{
  product: {
    slug: string
    name: string
    category: string
    price: string
    image: string
    shortDescription: string
    accent?: string
  }
  priceLabel: string
  variant?: 'featured' | 'latest' | 'trend' | 'listing'
}>()
</script>

<template>
  <NuxtLink
    :to="`/products/${product.slug}`"
    :class="{
      'featured-card': variant === 'featured',
      'latest-card': variant === 'latest',
      'trend-card': variant === 'trend',
      'listing-card': variant === 'listing',
    }"
  >
    <div
      :class="[
        variant === 'featured' ? 'featured-media' : '',
        variant === 'latest' ? 'latest-media' : '',
        variant === 'trend' ? 'trend-media' : '',
        variant === 'listing' ? 'listing-media' : '',
        variant === 'latest' && product.accent ? `accent-${product.accent}` : '',
      ]"
    >
      <img :src="product.image" :alt="product.name">
    </div>

    <div v-if="variant === 'featured'" class="featured-copy">
      <h3>{{ product.name }}</h3>
      <p>{{ product.shortDescription }}</p>
      <strong>{{ priceLabel }}</strong>
    </div>

    <template v-else-if="variant === 'latest'">
      <div class="latest-copy">
        <span>{{ product.name }}</span>
        <strong>{{ priceLabel }}</strong>
      </div>
      <p>{{ product.category }}</p>
    </template>

    <template v-else-if="variant === 'trend'">
      <h3>{{ product.name }}</h3>
      <p>{{ product.category }}</p>
      <strong>{{ priceLabel }}</strong>
    </template>

    <div v-else class="listing-copy">
      <span>{{ product.category }}</span>
      <h3>{{ product.name }}</h3>
      <p>{{ product.shortDescription }}</p>
      <strong>{{ priceLabel }}</strong>
    </div>
  </NuxtLink>
</template>
