<script setup lang="ts">
import type { CatalogProductDetailResponse } from '~/composables/useCatalogApi'

const route = useRoute()
const config = useRuntimeConfig()
const {
  allProducts,
  formatPrice,
} = useStorefrontContent()
const { mergeProductDetail } = useCatalogApi()
const { addItem } = useCart()

const slug = computed(() => String(route.params.slug))
const fallbackProduct = computed(() =>
  allProducts.find((item) => item.slug === slug.value) ?? allProducts[0],
)
const { data } = await useFetch<CatalogProductDetailResponse | null>(
  () => `/catalog/products/${slug.value}`,
  {
    baseURL: config.public.apiBaseUrl,
    default: () => null,
    server: false,
    watch: [slug],
  },
)

const product = computed(() =>
  data.value ? mergeProductDetail(data.value, fallbackProduct.value) : fallbackProduct.value,
)

const variants = computed(() => data.value?.variants ?? [])
const selectedVariantId = ref<string>('')
const selectedGalleryImage = ref('')

watch(variants, (value) => {
  selectedVariantId.value = value[0]?.id ?? ''
}, { immediate: true })

watch(product, (value) => {
  selectedGalleryImage.value = value.gallery[0] ?? value.image
}, { immediate: true })

const selectedVariant = computed(() =>
  variants.value.find((variant) => variant.id === selectedVariantId.value) ?? variants.value[0] ?? null,
)

const selectedPrice = computed(() =>
  selectedVariant.value?.price ? String(selectedVariant.value.price) : product.value.price,
)

const selectedCompareAtPrice = computed(() =>
  selectedVariant.value?.compare_at_price ? String(selectedVariant.value.compare_at_price) : '',
)

const selectedStock = computed(() => selectedVariant.value?.stock_quantity ?? 0)
const selectedSize = computed(() => selectedVariant.value?.size ?? '')
const selectedColor = computed(() => selectedVariant.value?.color ?? '')
const canAddToCart = computed(() => !selectedVariant.value || selectedStock.value > 0)

const relatedProducts = computed(() =>
  allProducts
    .filter((item) => item.slug !== product.value.slug && item.categorySlug === product.value.categorySlug)
    .slice(0, 3),
)

const addCurrentProductToCart = () => {
  addItem({
    lineId: selectedVariant.value?.id ?? product.value.slug,
    slug: product.value.slug,
    name: product.value.name,
    category: product.value.category,
    image: selectedGalleryImage.value || product.value.image,
    price: selectedPrice.value,
    variantId: selectedVariant.value?.id,
    variantName: selectedVariant.value?.name,
    sku: selectedVariant.value?.sku,
    size: selectedVariant.value?.size ?? undefined,
    color: selectedVariant.value?.color ?? undefined,
  })
}
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container product-detail">
        <div class="product-detail-media">
          <img :src="selectedGalleryImage || product.image" :alt="product.name">
          <div class="product-gallery-strip">
            <button
              v-for="image in product.gallery"
              :key="image"
              type="button"
              class="product-gallery-thumb"
              :class="{ 'is-active': selectedGalleryImage === image }"
              @click="selectedGalleryImage = image"
            >
              <img :src="image" :alt="product.name">
            </button>
          </div>
        </div>
        <div class="product-detail-copy">
          <p class="product-breadcrumbs">
            <NuxtLink to="/shop">Shop</NuxtLink>
            <span>/</span>
            <NuxtLink :to="`/collections/${product.categorySlug}`">{{ product.category }}</NuxtLink>
          </p>
          <p class="route-kicker">{{ product.category }}</p>
          <h1>{{ product.name }}</h1>
          <p class="route-copy">{{ product.shortDescription }}</p>
          <p class="product-story">{{ product.story }}</p>
          <div class="product-price-stack">
            <strong class="product-detail-price">{{ formatPrice(selectedPrice) }}</strong>
            <span v-if="selectedCompareAtPrice" class="product-compare-price">{{ formatPrice(selectedCompareAtPrice) }}</span>
          </div>
          <div v-if="variants.length" class="product-variant-shell">
            <label class="shop-field">
              <span>Choose Variant</span>
              <select v-model="selectedVariantId">
                <option v-for="variant in variants" :key="variant.id" :value="variant.id">
                  {{ variant.name }}<template v-if="variant.size"> / {{ variant.size }}</template><template v-if="variant.color"> / {{ variant.color }}</template>
                </option>
              </select>
            </label>
            <div class="product-variant-meta">
              <div>
                <span>SKU</span>
                <strong>{{ selectedVariant?.sku ?? 'N/A' }}</strong>
              </div>
              <div>
                <span>Size</span>
                <strong>{{ selectedSize || 'One Size' }}</strong>
              </div>
              <div>
                <span>Colour</span>
                <strong>{{ selectedColor || 'Seasonal' }}</strong>
              </div>
              <div>
                <span>Availability</span>
                <strong :class="{ 'is-low-stock': selectedStock <= 3 }">
                  {{ selectedStock > 0 ? `${selectedStock} in stock` : 'Out of stock' }}
                </strong>
              </div>
            </div>
          </div>
          <div class="product-specs">
            <div>
              <span>Materials</span>
              <strong>{{ product.materials.join(' • ') }}</strong>
            </div>
            <div>
              <span>Colours</span>
              <strong>{{ product.colors.join(' • ') }}</strong>
            </div>
            <div>
              <span>Sizes</span>
              <strong>{{ product.sizes.join(' • ') }}</strong>
            </div>
          </div>
          <div class="product-detail-actions">
            <button type="button" class="hero-cta product-action-button" :disabled="!canAddToCart" @click="addCurrentProductToCart">
              {{ canAddToCart ? 'Add To Cart' : 'Out Of Stock' }}
            </button>
            <NuxtLink to="/cart" class="secondary-link">
              View Cart
            </NuxtLink>
            <NuxtLink :to="`/collections/${product.categorySlug}`" class="secondary-link">
              View Collection
            </NuxtLink>
          </div>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container">
        <SectionHeading title="You May Also Like" />
        <div class="listing-grid">
          <ProductCard
            v-for="item in relatedProducts"
            :key="item.slug"
            :product="item"
            :price-label="formatPrice(item.price)"
            variant="listing"
          />
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
