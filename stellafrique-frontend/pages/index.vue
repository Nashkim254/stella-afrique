<script setup lang="ts">
const config = useRuntimeConfig()
const {
  storeName,
  latestTabs,
  fallbackFeaturedProducts,
  latestProducts,
  offerCards,
  trendProducts,
  categoryCards,
  blogCards,
  brandLogos,
  formatPrice,
} = useStorefrontContent()

const { data } = await useFetch<{ products: typeof fallbackFeaturedProducts }>('/catalog/featured', {
  baseURL: config.public.apiBaseUrl,
  default: () => ({ products: fallbackFeaturedProducts }),
  server: false,
})

const featuredProducts = computed(() =>
  (data.value?.products?.length ? data.value.products : fallbackFeaturedProducts).slice(0, 4),
)
</script>

<template>
  <main>
    <SiteHeader />

    <section class="hero-banner">
      <div class="container hero-banner-inner">
        <div class="hero-lamp">
          <span class="lamp-wire" />
          <span class="lamp-shade" />
        </div>

        <div class="hero-copy">
          <p class="hero-kicker">Best fashion picks for your wardrobe.</p>
          <h1>New clothing collection trends for now.</h1>
          <p class="hero-text">
            Discover elevated everyday pieces, soft knits, polished layers, and easy
            occasion looks built for a modern African wardrobe.
          </p>
          <NuxtLink to="/shop" class="hero-cta">Shop Now</NuxtLink>
        </div>

        <div class="hero-visual">
          <div class="hero-ring" />
          <div class="hero-discount">50%<span>off</span></div>
          <img src="/images/products/fashion-18.jpg" alt="Featured collection preview">
        </div>
      </div>
    </section>

    <section id="featured-products" class="section-block">
      <div class="container">
        <SectionHeading title="Featured Products" />
        <div class="featured-grid">
          <ProductCard
            v-for="product in featuredProducts"
            :key="product.id"
            :product="{
              slug: product.slug,
              name: product.name,
              category: product.category,
              price: product.price ?? '',
              image: product.primary_image_url ?? '',
              shortDescription: product.short_description ?? '',
            }"
            :price-label="formatPrice(product.price)"
            variant="featured"
          />
        </div>
      </div>
    </section>

    <section class="section-block latest-section">
      <div class="container">
        <SectionHeading title="Latest Products" :tabs="latestTabs" active-tab="New Arrival" />

        <div class="latest-grid">
          <ProductCard
            v-for="item in latestProducts"
            :key="item.id"
            :product="{
              slug: item.slug,
              name: item.name,
              category: item.category,
              price: item.price,
              image: item.image,
              shortDescription: `${item.category} piece chosen for a refined, wearable wardrobe.`,
              accent: item.accent,
            }"
            :price-label="formatPrice(item.price)"
            variant="latest"
          />
        </div>
      </div>
    </section>

    <section class="section-block offers-section">
      <div class="container">
        <SectionHeading :title="`What ${storeName} Offers`" />
        <div class="offer-grid">
          <article v-for="offer in offerCards" :key="offer.title" class="offer-card">
            <span class="offer-icon" />
            <h3>{{ offer.title }}</h3>
            <p>{{ offer.copy }}</p>
          </article>
        </div>
      </div>
    </section>

    <section class="story-banner">
      <div class="container story-inner">
        <div class="story-image">
          <img src="/images/products/fashion-02.jpg" alt="Editorial collection">
        </div>
        <div class="story-copy">
          <h2>Unique features of the latest and trending clothing</h2>
          <ul>
            <li>Soft palettes built around wearable wardrobe staples.</li>
            <li>Mix-and-match styling from knits to occasion layers.</li>
            <li>Pieces selected for gifting, dressing up, and everyday ease.</li>
          </ul>
          <div class="story-actions">
            <NuxtLink to="/products/soft-studio-layer" class="hero-cta">View Product</NuxtLink>
            <div>
              <strong>Soft Studio Knit</strong>
              <span>{{ formatPrice('4500') }}</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container">
        <SectionHeading title="Trending Products" />
        <div class="trend-grid">
          <ProductCard
            v-for="item in trendProducts"
            :key="item.id"
            :product="{
              slug: item.slug,
              name: item.name,
              category: item.category,
              price: item.price,
              image: item.image,
              shortDescription: `${item.category} look with elevated styling potential.`,
            }"
            :price-label="formatPrice(item.price)"
            variant="trend"
          />
        </div>

        <div class="promo-grid">
          <article class="promo-card promo-card-pink">
            <p>23% off in all products</p>
            <NuxtLink to="/collections/sale">Shop Now</NuxtLink>
          </article>
          <article class="promo-card promo-card-blue">
            <p>New season wardrobe edit</p>
            <NuxtLink to="/collections/new-in">View Collection</NuxtLink>
          </article>
          <article class="promo-list">
            <NuxtLink to="/products/daylight-knit" class="promo-mini">
              <img src="/images/products/fashion-08.jpg" alt="Mini look 1">
              <div>
                <strong>Minimal Set</strong>
                <span>{{ formatPrice('5200') }}</span>
              </div>
            </NuxtLink>
            <NuxtLink to="/products/soft-form-blazer" class="promo-mini">
              <img src="/images/products/fashion-11.jpg" alt="Mini look 2">
              <div>
                <strong>Evening Layer</strong>
                <span>{{ formatPrice('6100') }}</span>
              </div>
            </NuxtLink>
            <NuxtLink to="/products/cloudline-cardigan" class="promo-mini">
              <img src="/images/products/fashion-01.jpg" alt="Mini look 3">
              <div>
                <strong>Gift Knit</strong>
                <span>{{ formatPrice('4700') }}</span>
              </div>
            </NuxtLink>
          </article>
        </div>
      </div>
    </section>

    <section class="section-block categories-section">
      <div class="container">
        <SectionHeading title="Top Categories" />
        <div class="category-grid">
          <NuxtLink v-for="item in categoryCards" :key="item.name" :to="`/collections/${item.slug}`" class="category-card">
            <div class="category-media">
              <img :src="item.image" :alt="item.name">
            </div>
            <strong>{{ item.name }}</strong>
            <span>{{ item.count }}</span>
          </NuxtLink>
        </div>
      </div>
    </section>

    <section class="newsletter-banner">
      <div class="container newsletter-inner">
        <h2>Get latest updates by subscribing to our newsletter</h2>
        <NuxtLink to="/shop" class="hero-cta">Shop Now</NuxtLink>
      </div>
    </section>

    <section class="brand-strip">
      <div class="container brand-strip-inner">
        <img v-for="logo in brandLogos" :key="logo" :src="logo" alt="Brand logo">
      </div>
    </section>

    <section class="section-block blog-section">
      <div class="container">
        <SectionHeading title="Latest Blog" />
        <div class="blog-grid">
          <BlogTeaserCard v-for="post in blogCards" :key="post.slug" :post="post" compact />
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
