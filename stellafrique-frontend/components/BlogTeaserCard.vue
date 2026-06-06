<script setup lang="ts">
defineProps<{
  post: {
    slug: string
    title: string
    excerpt: string
    image: string
  }
  compact?: boolean
}>()
</script>

<template>
  <NuxtLink :to="`/blog/${post.slug}`" class="blog-card">
    <div class="blog-media">
      <img :src="post.image" :alt="post.title">
    </div>
    <div class="blog-copy">
      <p>Style Edit</p>
      <h3>{{ post.title }}</h3>
      <span>{{ compact ? 'Read More' : post.excerpt }}</span>
    </div>
  </NuxtLink>
</template>
