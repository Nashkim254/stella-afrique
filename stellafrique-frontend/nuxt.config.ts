export default defineNuxtConfig({
  devtools: { enabled: true },
  css: ['~/assets/css/main.css'],
  runtimeConfig: {
    public: {
      apiBaseUrl: process.env.NUXT_PUBLIC_API_BASE_URL ?? 'http://localhost:8080/api/v1',
    },
  },
  app: {
    head: {
      title: 'Stellafrique',
      meta: [
        {
          name: 'description',
          content: 'Modern fashion storefront for curated womenswear and everyday essentials.',
        },
      ],
    },
  },
  compatibilityDate: '2025-01-15',
})

