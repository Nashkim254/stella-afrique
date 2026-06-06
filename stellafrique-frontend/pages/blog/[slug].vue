<script setup lang="ts">
const route = useRoute()
const {
  blogCards,
} = useStorefrontContent()

const post = computed(() =>
  blogCards.find((item) => item.slug === String(route.params.slug)) ?? blogCards[0],
)
</script>

<template>
  <main>
    <SiteHeader />

    <section class="route-hero">
      <div class="container article-layout">
        <div class="article-media">
          <img :src="post.image" :alt="post.title">
        </div>
        <div class="article-copy">
          <p class="route-kicker">Style Edit</p>
          <h1>{{ post.title }}</h1>
          <p class="route-copy">{{ post.excerpt }}</p>
          <p class="article-body">
            Focus on wearable colour palettes, thoughtful layering, and fewer better pieces.
            This page is the first route scaffold for the editorial side of the storefront and
            can later be driven by backend content instead of local mock data.
          </p>
        </div>
      </div>
    </section>

    <SiteFooter />
  </main>
</template>
