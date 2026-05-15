<script setup lang="ts">
type FeaturedProduct = {
  id: string
  name: string
  slug: string
  description?: string | null
  brand?: string | null
  default_price?: string | null
  primary_image_url?: string | null
  available_sizes: string[]
  available_colors: string[]
}

type StorefrontHome = {
  store: {
    store_name: string
    currency: string
    delivery_fee: string
    paybill_number: string
  }
  featured_products: FeaturedProduct[]
}

const config = useRuntimeConfig()

const fallback: StorefrontHome = {
  store: {
    store_name: 'Stellafrique',
    currency: 'KES',
    delivery_fee: '200',
    paybill_number: '',
  },
  featured_products: [
    {
      id: 'sample-1',
      name: 'Tailored Linen Set',
      slug: 'tailored-linen-set',
      description: 'Lightweight two-piece look built for warm city days.',
      brand: 'Stellafrique Studio',
      default_price: '5400.00',
      primary_image_url: null,
      available_sizes: ['S', 'M', 'L'],
      available_colors: ['Ivory', 'Clay'],
    },
    {
      id: 'sample-2',
      name: 'Structured Weekend Dress',
      slug: 'structured-weekend-dress',
      description: 'Clean silhouette with a soft drape and practical pockets.',
      brand: 'Stellafrique Studio',
      default_price: '6200.00',
      primary_image_url: null,
      available_sizes: ['8', '10', '12'],
      available_colors: ['Black', 'Olive'],
    },
  ],
}

const { data } = await useFetch<StorefrontHome>('/storefront/home', {
  baseURL: config.public.apiBaseUrl,
  default: () => fallback,
  server: false,
})

const storefront = computed(() => data.value ?? fallback)

const formatPrice = (value?: string | null) => {
  if (!value) {
    return 'Price on request'
  }

  return new Intl.NumberFormat('en-KE', {
    style: 'currency',
    currency: storefront.value.store.currency || 'KES',
    maximumFractionDigits: 0,
  }).format(Number(value))
}
</script>

<template>
  <main class="page-shell">
    <section class="hero">
      <div class="hero-copy">
        <p class="eyebrow">Curated Everyday Style</p>
        <h1>{{ storefront.store.store_name }}</h1>
        <p class="lead">
          A fashion storefront for confident, wearable pieces. This is the base Nuxt shell
          we’ll replace with your Figma layouts once you share them.
        </p>

        <div class="hero-actions">
          <a href="#featured" class="primary-link">Shop Featured</a>
          <span class="delivery-note">
            Delivery from {{ storefront.store.currency }} {{ storefront.store.delivery_fee }}
          </span>
        </div>
      </div>

      <div class="hero-card">
        <p>Storefront API</p>
        <strong>/api/v1/storefront/home</strong>
        <span>Powered by Axum + SeaORM + Postgres</span>
      </div>
    </section>

    <section id="featured" class="featured-section">
      <div class="section-heading">
        <p class="eyebrow">Featured Products</p>
        <h2>Use this as the first storefront slice</h2>
      </div>

      <div class="product-grid">
        <article v-for="product in storefront.featured_products" :key="product.id" class="product-card">
          <div class="product-media">
            <img
              v-if="product.primary_image_url"
              :src="product.primary_image_url"
              :alt="product.name"
            >
            <div v-else class="image-fallback">{{ product.name.slice(0, 1) }}</div>
          </div>

          <div class="product-copy">
            <p class="product-brand">{{ product.brand || storefront.store.store_name }}</p>
            <h3>{{ product.name }}</h3>
            <p>{{ product.description }}</p>
            <strong>{{ formatPrice(product.default_price) }}</strong>
          </div>

          <div class="product-meta">
            <span>{{ product.available_sizes.join(' / ') || 'One size' }}</span>
            <span>{{ product.available_colors.join(' / ') || 'Seasonal colorways' }}</span>
          </div>
        </article>
      </div>
    </section>
  </main>
</template>

