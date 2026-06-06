<script setup lang="ts">
import type { CatalogProductsResponse } from '~/composables/useCatalogApi'

const route = useRoute()
const config = useRuntimeConfig()
const {
  categoryCards,
  allProducts,
  formatPrice,
} = useStorefrontContent()
const { mapCatalogCardToProductRecord } = useCatalogApi()

const slug = computed(() => String(route.params.slug))
const { data } = await useFetch<CatalogProductsResponse>(
  () => `/catalog/collections/${slug.value}`,
  {
    baseURL: config.public.apiBaseUrl,
    default: () => ({ products: [] }),
    server: false,
    watch: [slug],
  },
)

const category = computed(() =>
  categoryCards.find((item) => item.slug === slug.value) ?? {
    name: slug.value.replace(/-/g, ' '),
    count: '',
    image: '/images/products/fashion-01.jpg',
  },
)
const products = computed(() =>
  data.value?.products?.length
    ? data.value.products.map((product) =>
        mapCatalogCardToProductRecord(
          product,
          allProducts.find((item) => item.slug === product.slug),
        ),
      )
    : allProducts.filter((item) => item.categorySlug === slug.value || slug.value === 'sale' || slug.value === 'new-in'),
)
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container route-hero-inner route-hero-split">
        <div>
          <p class="route-kicker">Collection</p>
          <h1>{{ category.name }}</h1>
          <p class="route-copy">
            Browse the {{ category.name.toLowerCase() }} edit and discover curated looks
            matched to the storefront’s current styling direction.
          </p>
        </div>
        <div class="route-hero-media">
          <img :src="category.image" :alt="category.name">
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container">
        <div class="listing-grid">
          <ProductCard
            v-for="product in products"
            :key="product.slug"
            :product="product"
            :price-label="formatPrice(product.price)"
            variant="listing"
          />
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
