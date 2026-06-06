<script setup lang="ts">
const {
  blogCards,
} = useStorefrontContent()
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container route-hero-inner">
        <div>
          <p class="route-kicker">Journal</p>
          <h1>Stories, styling notes, and wardrobe guidance.</h1>
          <p class="route-copy">
            Editorial notes from the Stellafrique storefront on dressing, gifting,
            and building a smaller better wardrobe.
          </p>
        </div>
      </div>
    </section>

    <section class="section-block">
      <div class="container">
        <div class="blog-grid">
          <BlogTeaserCard v-for="post in blogCards" :key="post.slug" :post="post" />
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
