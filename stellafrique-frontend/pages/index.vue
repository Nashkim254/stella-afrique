<script setup lang="ts">
type FeaturedProduct = {
  id: string
  category_id?: string | null
  name: string
  slug: string
  short_description?: string | null
  price?: string | null
  primary_image_url?: string | null
}

type CatalogFeatured = {
  products: FeaturedProduct[]
}

const config = useRuntimeConfig()
const storeName = 'Stellafrique'
const currency = 'KES'

const fallback: CatalogFeatured = {
  products: [
    {
      id: 'sample-1',
      name: 'Tailored Linen Set',
      slug: 'tailored-linen-set',
      short_description: 'Lightweight two-piece look built for warm city days.',
      price: '5400.00',
      primary_image_url: null,
    },
    {
      id: 'sample-2',
      name: 'Structured Weekend Dress',
      slug: 'structured-weekend-dress',
      short_description: 'Clean silhouette with a soft drape and practical pockets.',
      price: '6200.00',
      primary_image_url: null,
    },
  ],
}

const { data } = await useFetch<CatalogFeatured>('/catalog/featured', {
  baseURL: config.public.apiBaseUrl,
  default: () => fallback,
  server: false,
})

const catalog = computed(() => data.value ?? fallback)

const formatPrice = (value?: string | null) => {
  if (!value) {
    return 'Price on request'
  }

  return new Intl.NumberFormat('en-KE', {
    style: 'currency',
    currency,
    maximumFractionDigits: 0,
  }).format(Number(value))
}
</script>

<template>
  <main class="page-shell">
    <section class="hero">
      <div class="hero-copy">
        <p class="eyebrow">Curated Everyday Style</p>
        <h1>{{ storeName }}</h1>
        <p class="lead">
          A fashion storefront for confident, wearable pieces. This is the base Nuxt shell
          we’ll replace with your Figma layouts once you share them.
        </p>

        <div class="hero-actions">
          <a href="#featured" class="primary-link">Shop Featured</a>
          <span class="delivery-note">Fresh catalog schema powered by SeaORM migrations</span>
        </div>
      </div>

      <div class="hero-card">
        <p>Catalog API</p>
        <strong>/api/v1/catalog/featured</strong>
        <span>Powered by Axum + SeaORM + Postgres</span>
      </div>
    </section>

    <section id="featured" class="featured-section">
      <div class="section-heading">
        <p class="eyebrow">Featured Products</p>
        <h2>Use this as the first storefront slice</h2>
      </div>

      <div class="product-grid">
        <article v-for="product in catalog.products" :key="product.id" class="product-card">
          <div class="product-media">
            <img
              v-if="product.primary_image_url"
              :src="product.primary_image_url"
              :alt="product.name"
            >
            <div v-else class="image-fallback">{{ product.name.slice(0, 1) }}</div>
          </div>

          <div class="product-copy">
            <p class="product-brand">{{ storeName }}</p>
            <h3>{{ product.name }}</h3>
            <p>{{ product.short_description }}</p>
            <strong>{{ formatPrice(product.price) }}</strong>
          </div>
        </article>
      </div>
    </section>
  </main>
</template>
