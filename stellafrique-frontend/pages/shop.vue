<script setup lang="ts">
import type { CatalogProductsResponse } from '~/composables/useCatalogApi'

const config = useRuntimeConfig()
const {
  allProducts,
  collectionOptions,
  formatPrice,
} = useStorefrontContent()
const { mapCatalogCardToProductRecord } = useCatalogApi()

const { data } = await useFetch<CatalogProductsResponse>('/catalog/products', {
  baseURL: config.public.apiBaseUrl,
  default: () => ({ products: [] }),
  server: false,
})

const selectedCollection = ref('all')
const selectedSort = ref('featured')
const searchTerm = ref('')

const sourceProducts = computed(() =>
  data.value?.products?.length
    ? data.value.products.map((product) =>
        mapCatalogCardToProductRecord(
          product,
          allProducts.find((item) => item.slug === product.slug),
        ),
      )
    : allProducts,
)

const filteredProducts = computed(() => {
  const term = searchTerm.value.trim().toLowerCase()

  let items = sourceProducts.value.filter((product) => {
    const matchesCollection = selectedCollection.value === 'all' || product.categorySlug === selectedCollection.value
    const matchesSearch = !term
      || product.name.toLowerCase().includes(term)
      || product.category.toLowerCase().includes(term)
      || product.shortDescription.toLowerCase().includes(term)

    return matchesCollection && matchesSearch
  })

  if (selectedSort.value === 'price-low') {
    items = [...items].sort((a, b) => Number(a.price) - Number(b.price))
  }
  else if (selectedSort.value === 'price-high') {
    items = [...items].sort((a, b) => Number(b.price) - Number(a.price))
  }
  else if (selectedSort.value === 'name') {
    items = [...items].sort((a, b) => a.name.localeCompare(b.name))
  }

  return items
})
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container route-hero-inner">
        <div>
          <p class="route-kicker">Shop All</p>
          <h1>Clothing and accessories chosen for everyday style.</h1>
          <p class="route-copy">
            Browse the full storefront edit across dresses, knitwear, accessories,
            outerwear, and easy gift picks.
          </p>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container">
        <div class="shop-toolbar">
          <div class="shop-toolbar-copy">
            <strong>{{ filteredProducts.length }} styles</strong>
            <span>Filter the full edit by collection, search, or price.</span>
          </div>

          <div class="shop-controls">
            <label class="shop-field">
              <span>Search</span>
              <input v-model="searchTerm" type="text" placeholder="Search looks, knits, dresses">
            </label>

            <label class="shop-field">
              <span>Collection</span>
              <select v-model="selectedCollection">
                <option value="all">All collections</option>
                <option v-for="option in collectionOptions" :key="option.value" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>

            <label class="shop-field">
              <span>Sort</span>
              <select v-model="selectedSort">
                <option value="featured">Featured</option>
                <option value="price-low">Price: Low to High</option>
                <option value="price-high">Price: High to Low</option>
                <option value="name">Name</option>
              </select>
            </label>
          </div>
        </div>

        <div class="listing-grid">
          <ProductCard
            v-for="product in filteredProducts"
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
