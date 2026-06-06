<script setup lang="ts">
const config = useRuntimeConfig()

type AdminOverview = {
  categories: Array<{
    id: string
    name: string
    slug: string
    description?: string | null
    image_url?: string | null
    sort_order: number
    is_active: boolean
  }>
  products: Array<{
    id: string
    name: string
    slug: string
    category?: string | null
    category_slug?: string | null
    short_description?: string | null
    description?: string | null
    status: string
    is_featured: boolean
  }>
}

type AdminProductDetail = {
  id: string
  name: string
  slug: string
  category?: string | null
  category_slug?: string | null
  short_description?: string | null
  description?: string | null
  status: string
  is_featured: boolean
  primary_image_url?: string | null
  images: Array<{
    image_url: string
    alt_text?: string | null
    is_primary: boolean
    sort_order: number
  }>
  variants: Array<{
    id: string
    name: string
    sku: string
    size?: string | null
    color?: string | null
    price: string | number
    compare_at_price?: string | number | null
    stock_quantity: number
    is_active: boolean
  }>
}

type ProductVariantDraft = {
  id?: string
  name: string
  sku: string
  size: string
  color: string
  price: string
  compare_at_price: string
  stock_quantity: string
  is_active: boolean
}

const categoryForm = reactive({
  name: '',
  slug: '',
  description: '',
  image_url: '',
  sort_order: '0',
})

const productForm = reactive({
  category_slug: '',
  name: '',
  slug: '',
  short_description: '',
  description: '',
  primary_image_url: '',
  gallery_image_urls: [] as string[],
  is_featured: false,
  variants: [
    {
      name: 'Default Variant',
      sku: '',
      size: 'M',
      color: '',
      price: '',
      compare_at_price: '',
      stock_quantity: '0',
    },
  ] as Array<{
    name: string
    sku: string
    size: string
    color: string
    price: string
    compare_at_price: string
    stock_quantity: string
  }>,
})

const feedback = ref('')
const isSubmittingCategory = ref(false)
const isSubmittingProduct = ref(false)
const isUploadingImage = ref(false)
const uploadingProductId = ref('')
const loadingProductId = ref('')
const editingCategoryId = ref('')
const editingProductId = ref('')
const deletingId = ref('')
const adminFetchOptions = {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include' as const,
}

const { data, refresh } = await useFetch<AdminOverview>('/admin/catalog/overview', {
  ...adminFetchOptions,
  default: () => ({ categories: [], products: [] }),
  server: false,
})

const categoryDrafts = reactive<Record<string, {
  name: string
  slug: string
  description: string
  image_url: string
  sort_order: string
  is_active: boolean
}>>({})

const productDrafts = reactive<Record<string, {
  category_slug: string
  name: string
  slug: string
  short_description: string
  description: string
  status: string
  is_featured: boolean
  primary_image_url: string
  gallery_image_urls: string[]
  variants: ProductVariantDraft[]
  detailLoaded: boolean
}>>({})

const categoryOptions = computed(() =>
  data.value?.categories ?? [],
)

function createEmptyVariant() {
  return {
    id: undefined,
    name: '',
    sku: '',
    size: '',
    color: '',
    price: '',
    compare_at_price: '',
    stock_quantity: '0',
    is_active: true,
  }
}

watch(
  data,
  (value) => {
    for (const category of value?.categories ?? []) {
      categoryDrafts[category.id] = {
        name: category.name,
        slug: category.slug,
        description: category.description ?? '',
        image_url: category.image_url ?? '',
        sort_order: String(category.sort_order),
        is_active: category.is_active,
      }
    }

    for (const product of value?.products ?? []) {
      productDrafts[product.id] = {
        category_slug: product.category_slug ?? '',
        name: product.name,
        slug: product.slug,
        short_description: product.short_description ?? '',
        description: product.description ?? '',
        status: product.status,
        is_featured: product.is_featured,
        primary_image_url: productDrafts[product.id]?.primary_image_url ?? '',
        gallery_image_urls: productDrafts[product.id]?.gallery_image_urls ?? [],
        variants: productDrafts[product.id]?.variants ?? [createEmptyVariant()],
        detailLoaded: productDrafts[product.id]?.detailLoaded ?? false,
      }
    }
  },
  { immediate: true },
)

const submitCategory = async () => {
  isSubmittingCategory.value = true
  feedback.value = ''

  try {
    await $fetch('/admin/categories', {
      method: 'POST',
      ...adminFetchOptions,
      body: {
        name: categoryForm.name,
        slug: categoryForm.slug,
        description: categoryForm.description || null,
        image_url: categoryForm.image_url || null,
        sort_order: Number(categoryForm.sort_order) || 0,
      },
    })

    Object.assign(categoryForm, {
      name: '',
      slug: '',
      description: '',
      image_url: '',
      sort_order: '0',
    })

    feedback.value = 'Category created.'
    await refresh()
  }
  finally {
    isSubmittingCategory.value = false
  }
}

const submitProduct = async () => {
  isSubmittingProduct.value = true
  feedback.value = ''

  try {
    await $fetch('/admin/products', {
      method: 'POST',
      ...adminFetchOptions,
      body: {
        category_slug: productForm.category_slug || null,
        name: productForm.name,
        slug: productForm.slug,
        short_description: productForm.short_description || null,
        description: productForm.description || null,
        primary_image_url: productForm.primary_image_url || null,
        gallery_image_urls: productForm.gallery_image_urls,
        price: productForm.variants[0]?.price || '0',
        compare_at_price: productForm.variants[0]?.compare_at_price || null,
        sku: productForm.variants[0]?.sku || '',
        variant_name: productForm.variants[0]?.name || 'Default Variant',
        size: productForm.variants[0]?.size || null,
        color: productForm.variants[0]?.color || null,
        stock_quantity: Number(productForm.variants[0]?.stock_quantity || 0) || 0,
        variants: productForm.variants.map((variant) => ({
          name: variant.name || 'Default Variant',
          sku: variant.sku,
          size: variant.size || null,
          color: variant.color || null,
          price: variant.price,
          compare_at_price: variant.compare_at_price || null,
          stock_quantity: Number(variant.stock_quantity) || 0,
        })),
        is_featured: productForm.is_featured,
      },
    })

    Object.assign(productForm, {
      category_slug: '',
      name: '',
      slug: '',
      short_description: '',
      description: '',
      primary_image_url: '',
      gallery_image_urls: [],
      is_featured: false,
      variants: [
        {
          name: 'Default Variant',
          sku: '',
          size: 'M',
          color: '',
          price: '',
          compare_at_price: '',
          stock_quantity: '0',
        },
      ],
    })

    feedback.value = 'Product created.'
    await refresh()
  }
  finally {
    isSubmittingProduct.value = false
  }
}

const uploadImage = async (event: Event, productId = '') => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]

  if (!file) {
    return
  }

  isUploadingImage.value = true
  uploadingProductId.value = productId
  feedback.value = ''

  try {
    const formData = new FormData()
    formData.append('file', file)

    const response = await $fetch<{ public_url: string }>('/admin/uploads/product-image', {
      method: 'POST',
      ...adminFetchOptions,
      body: formData,
    })

    if (productId) {
      const draft = productDrafts[productId]
      if (!draft.primary_image_url) {
        draft.primary_image_url = response.public_url
      }
      if (!draft.gallery_image_urls.includes(response.public_url)) {
        draft.gallery_image_urls.push(response.public_url)
      }
    } else {
      if (!productForm.primary_image_url) {
        productForm.primary_image_url = response.public_url
      }

      if (!productForm.gallery_image_urls.includes(response.public_url)) {
        productForm.gallery_image_urls.push(response.public_url)
      }
    }

    feedback.value = 'Image uploaded to Supabase Storage and added to the gallery.'
  }
  finally {
    isUploadingImage.value = false
    uploadingProductId.value = ''
    input.value = ''
  }
}

const removeGalleryImage = (imageUrl: string) => {
  productForm.gallery_image_urls = productForm.gallery_image_urls.filter((url) => url !== imageUrl)

  if (productForm.primary_image_url === imageUrl) {
    productForm.primary_image_url = productForm.gallery_image_urls[0] ?? ''
  }
}

const setPrimaryImage = (imageUrl: string) => {
  productForm.primary_image_url = imageUrl
  productForm.gallery_image_urls = [
    imageUrl,
    ...productForm.gallery_image_urls.filter((url) => url !== imageUrl),
  ]
}

const addVariant = () => {
  productForm.variants.push(createEmptyVariant())
}

const removeVariant = (index: number) => {
  if (productForm.variants.length === 1) {
    productForm.variants[0] = createEmptyVariant()
    return
  }

  productForm.variants.splice(index, 1)
}

const removeVariantDraft = (productId: string, index: number) => {
  const draft = productDrafts[productId]
  if (draft.variants.length === 1) {
    draft.variants[0] = createEmptyVariant()
    return
  }

  draft.variants.splice(index, 1)
}

const addVariantDraft = (productId: string) => {
  productDrafts[productId].variants.push(createEmptyVariant())
}

const removeGalleryImageFromDraft = (productId: string, imageUrl: string) => {
  const draft = productDrafts[productId]
  draft.gallery_image_urls = draft.gallery_image_urls.filter((url) => url !== imageUrl)

  if (draft.primary_image_url === imageUrl) {
    draft.primary_image_url = draft.gallery_image_urls[0] ?? ''
  }
}

const setPrimaryImageForDraft = (productId: string, imageUrl: string) => {
  const draft = productDrafts[productId]
  draft.primary_image_url = imageUrl
  draft.gallery_image_urls = [
    imageUrl,
    ...draft.gallery_image_urls.filter((url) => url !== imageUrl),
  ]
}

const openProductEditor = async (productId: string) => {
  if (editingProductId.value === productId) {
    editingProductId.value = ''
    return
  }

  editingProductId.value = productId

  if (productDrafts[productId]?.detailLoaded) {
    return
  }

  loadingProductId.value = productId

  try {
    const detail = await $fetch<AdminProductDetail>(`/admin/products/${productId}`, {
      ...adminFetchOptions,
    })

    productDrafts[productId] = {
      category_slug: detail.category_slug ?? '',
      name: detail.name,
      slug: detail.slug,
      short_description: detail.short_description ?? '',
      description: detail.description ?? '',
      status: detail.status,
      is_featured: detail.is_featured,
      primary_image_url: detail.primary_image_url ?? '',
      gallery_image_urls: detail.images.map((image) => image.image_url),
      variants: detail.variants.length
        ? detail.variants.map((variant) => ({
            id: variant.id,
            name: variant.name,
            sku: variant.sku,
            size: variant.size ?? '',
            color: variant.color ?? '',
            price: String(variant.price),
            compare_at_price: variant.compare_at_price ? String(variant.compare_at_price) : '',
            stock_quantity: String(variant.stock_quantity),
            is_active: variant.is_active,
          }))
        : [createEmptyVariant()],
      detailLoaded: true,
    }
  }
  finally {
    loadingProductId.value = ''
  }
}

const saveCategory = async (categoryId: string) => {
  feedback.value = ''

  await $fetch(`/admin/categories/${categoryId}`, {
    method: 'PATCH',
    ...adminFetchOptions,
    body: {
      name: categoryDrafts[categoryId].name,
      slug: categoryDrafts[categoryId].slug,
      description: categoryDrafts[categoryId].description,
      image_url: categoryDrafts[categoryId].image_url,
      sort_order: Number(categoryDrafts[categoryId].sort_order) || 0,
      is_active: categoryDrafts[categoryId].is_active,
    },
  })

  feedback.value = 'Category updated.'
  editingCategoryId.value = ''
  await refresh()
}

const deleteCategory = async (categoryId: string) => {
  if (!window.confirm('Delete this category? Products linked to it will become unassigned.')) {
    return
  }

  deletingId.value = categoryId
  feedback.value = ''

  try {
    await $fetch(`/admin/categories/${categoryId}`, {
      method: 'DELETE',
      ...adminFetchOptions,
    })

    feedback.value = 'Category deleted.'
    if (editingCategoryId.value === categoryId) {
      editingCategoryId.value = ''
    }
    await refresh()
  }
  finally {
    deletingId.value = ''
  }
}

const saveProduct = async (productId: string) => {
  feedback.value = ''
  const draft = productDrafts[productId]

  await $fetch(`/admin/products/${productId}`, {
    method: 'PATCH',
    ...adminFetchOptions,
    body: {
      category_slug: draft.category_slug,
      name: draft.name,
      slug: draft.slug,
      short_description: draft.short_description,
      description: draft.description,
      status: draft.status,
      is_featured: draft.is_featured,
      primary_image_url: draft.primary_image_url || null,
      gallery_image_urls: draft.gallery_image_urls,
      variants: draft.variants.map((variant) => ({
        id: variant.id || null,
        name: variant.name || 'Default Variant',
        sku: variant.sku,
        size: variant.size || null,
        color: variant.color || null,
        price: variant.price,
        compare_at_price: variant.compare_at_price || null,
        stock_quantity: Number(variant.stock_quantity) || 0,
      })),
    },
  })

  feedback.value = 'Product updated.'
  draft.detailLoaded = false
  editingProductId.value = ''
  await refresh()
}

const deleteProduct = async (productId: string) => {
  if (!window.confirm('Delete this product? This only works if it has never appeared in customer orders.')) {
    return
  }

  deletingId.value = productId
  feedback.value = ''

  try {
    await $fetch(`/admin/products/${productId}`, {
      method: 'DELETE',
      ...adminFetchOptions,
    })

    feedback.value = 'Product deleted.'
    if (editingProductId.value === productId) {
      editingProductId.value = ''
    }
    await refresh()
  }
  finally {
    deletingId.value = ''
  }
}
</script>

<template>
  <AdminShell
    title="Catalog management for Stellafrique."
    description="Create categories and products directly against the Rust API while the full admin suite keeps expanding."
  >
    <section class="admin-content-section">
      <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>
      <div class="admin-grid">
        <div class="admin-panel">
          <h2>Create Category</h2>
          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Name</span>
              <input v-model="categoryForm.name" type="text" placeholder="Knitwear">
            </label>
            <label class="shop-field">
              <span>Slug</span>
              <input v-model="categoryForm.slug" type="text" placeholder="knitwear">
            </label>
            <label class="shop-field">
              <span>Image URL</span>
              <input v-model="categoryForm.image_url" type="text" placeholder="/images/products/fashion-06.jpg">
            </label>
            <label class="shop-field">
              <span>Sort Order</span>
              <input v-model="categoryForm.sort_order" type="number" min="0">
            </label>
            <label class="shop-field admin-field-span">
              <span>Description</span>
              <textarea v-model="categoryForm.description" rows="4" placeholder="Soft layers, cardigans, and elevated knit staples." />
            </label>
          </div>
          <button type="button" class="hero-cta admin-submit" :disabled="isSubmittingCategory" @click="submitCategory">
            {{ isSubmittingCategory ? 'Creating...' : 'Create Category' }}
          </button>
        </div>

        <div class="admin-panel">
          <h2>Create Product</h2>
          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Category</span>
              <select v-model="productForm.category_slug">
                <option value="">No category</option>
                <option v-for="category in categoryOptions" :key="category.id" :value="category.slug">
                  {{ category.name }}
                </option>
              </select>
            </label>
            <label class="shop-field">
              <span>Name</span>
              <input v-model="productForm.name" type="text" placeholder="Soft Studio Layer">
            </label>
            <label class="shop-field">
              <span>Slug</span>
              <input v-model="productForm.slug" type="text" placeholder="soft-studio-layer">
            </label>
            <label class="shop-field">
              <span>Upload Images</span>
              <input type="file" accept="image/*" @change="uploadImage">
            </label>
            <label class="shop-field">
              <span>Primary Image URL</span>
              <input
                v-model="productForm.primary_image_url"
                type="text"
                :placeholder="isUploadingImage ? 'Uploading to Supabase...' : 'https://...'"
              >
            </label>
            <div v-if="productForm.gallery_image_urls.length" class="admin-field-span admin-gallery-shell">
              <div class="admin-gallery-header">
                <span>Gallery Preview</span>
                <small>The first image is used as the primary storefront image.</small>
              </div>
              <div class="admin-gallery-grid">
                <article
                  v-for="imageUrl in productForm.gallery_image_urls"
                  :key="imageUrl"
                  class="admin-gallery-card"
                  :class="{ 'is-primary': productForm.primary_image_url === imageUrl }"
                >
                  <div class="admin-gallery-media">
                    <img :src="imageUrl" alt="" loading="lazy">
                  </div>
                  <div class="admin-gallery-actions">
                    <button type="button" class="secondary-link admin-gallery-button" @click="setPrimaryImage(imageUrl)">
                      {{ productForm.primary_image_url === imageUrl ? 'Primary' : 'Make Primary' }}
                    </button>
                    <button type="button" class="secondary-link admin-gallery-button admin-gallery-remove" @click="removeGalleryImage(imageUrl)">
                      Remove
                    </button>
                  </div>
                </article>
              </div>
            </div>
            <label class="shop-field admin-checkbox">
              <span>Featured</span>
              <input v-model="productForm.is_featured" type="checkbox">
            </label>
            <div class="admin-field-span admin-variants-shell">
              <div class="admin-variants-header">
                <div>
                  <span>Variants</span>
                  <small>Create size, color, and price combinations before saving the product.</small>
                </div>
                <button type="button" class="secondary-link admin-variant-add" @click="addVariant">
                  Add Variant
                </button>
              </div>

              <div class="admin-variant-list">
                <article
                  v-for="(variant, index) in productForm.variants"
                  :key="`${variant.sku}-${index}`"
                  class="admin-variant-card"
                >
                  <div class="admin-variant-card-top">
                    <strong>Variant {{ index + 1 }}</strong>
                    <button type="button" class="secondary-link admin-variant-remove" @click="removeVariant(index)">
                      Remove
                    </button>
                  </div>
                  <div class="admin-form-grid">
                    <label class="shop-field">
                      <span>Name</span>
                      <input v-model="variant.name" type="text" placeholder="Stone / M">
                    </label>
                    <label class="shop-field">
                      <span>SKU</span>
                      <input v-model="variant.sku" type="text" placeholder="STL-OUT-STU-M">
                    </label>
                    <label class="shop-field">
                      <span>Size</span>
                      <input v-model="variant.size" type="text" placeholder="M">
                    </label>
                    <label class="shop-field">
                      <span>Color</span>
                      <input v-model="variant.color" type="text" placeholder="Stone">
                    </label>
                    <label class="shop-field">
                      <span>Price</span>
                      <input v-model="variant.price" type="text" placeholder="6900">
                    </label>
                    <label class="shop-field">
                      <span>Compare At</span>
                      <input v-model="variant.compare_at_price" type="text" placeholder="7600">
                    </label>
                    <label class="shop-field">
                      <span>Stock</span>
                      <input v-model="variant.stock_quantity" type="number" min="0">
                    </label>
                  </div>
                </article>
              </div>
            </div>
            <label class="shop-field admin-field-span">
              <span>Short Description</span>
              <textarea v-model="productForm.short_description" rows="3" placeholder="An easy outer layer designed for neat, minimal outfits." />
            </label>
            <label class="shop-field admin-field-span">
              <span>Description</span>
              <textarea v-model="productForm.description" rows="5" placeholder="Longer product story for the detail page." />
            </label>
          </div>
          <button type="button" class="hero-cta admin-submit" :disabled="isSubmittingProduct" @click="submitProduct">
            {{ isSubmittingProduct ? 'Creating...' : 'Create Product' }}
          </button>
        </div>
      </div>
    </section>

    <section class="admin-content-section admin-listing-section">
      <div class="admin-grid">
        <div class="admin-panel">
          <h2>Categories</h2>
          <div class="admin-table">
            <article v-for="category in categoryOptions" :key="category.id" class="admin-row">
              <div>
                <strong>{{ category.name }}</strong>
                <p>{{ category.slug }}</p>
              </div>
              <div class="admin-inline-actions">
                <span>#{{ category.sort_order }}</span>
                <button type="button" class="secondary-link admin-inline-button" @click="editingCategoryId = editingCategoryId === category.id ? '' : category.id">
                  {{ editingCategoryId === category.id ? 'Close' : 'Edit' }}
                </button>
                <button
                  type="button"
                  class="secondary-link admin-inline-button admin-inline-delete"
                  :disabled="deletingId === category.id"
                  @click="deleteCategory(category.id)"
                >
                  {{ deletingId === category.id ? 'Deleting...' : 'Delete' }}
                </button>
              </div>
              <div v-if="editingCategoryId === category.id" class="admin-inline-editor">
                <div class="admin-form-grid">
                  <label class="shop-field">
                    <span>Name</span>
                    <input v-model="categoryDrafts[category.id].name" type="text">
                  </label>
                  <label class="shop-field">
                    <span>Slug</span>
                    <input v-model="categoryDrafts[category.id].slug" type="text">
                  </label>
                  <label class="shop-field">
                    <span>Image URL</span>
                    <input v-model="categoryDrafts[category.id].image_url" type="text">
                  </label>
                  <label class="shop-field">
                    <span>Sort Order</span>
                    <input v-model="categoryDrafts[category.id].sort_order" type="number" min="0">
                  </label>
                  <label class="shop-field admin-checkbox">
                    <span>Active</span>
                    <input v-model="categoryDrafts[category.id].is_active" type="checkbox">
                  </label>
                  <label class="shop-field admin-field-span">
                    <span>Description</span>
                    <textarea v-model="categoryDrafts[category.id].description" rows="3" />
                  </label>
                </div>
                <button type="button" class="hero-cta admin-submit" @click="saveCategory(category.id)">
                  Save Category
                </button>
              </div>
            </article>
          </div>
        </div>

        <div class="admin-panel">
          <h2>Products</h2>
          <div class="admin-table">
            <article v-for="product in data?.products ?? []" :key="product.id" class="admin-row">
              <div>
                <strong>{{ product.name }}</strong>
                <p>{{ product.slug }} · {{ product.category || 'Unassigned' }}</p>
              </div>
              <div class="admin-inline-actions">
                <span>{{ product.is_featured ? 'Featured' : product.status }}</span>
                <button type="button" class="secondary-link admin-inline-button" @click="openProductEditor(product.id)">
                  {{ editingProductId === product.id ? 'Close' : 'Edit' }}
                </button>
                <button
                  type="button"
                  class="secondary-link admin-inline-button admin-inline-delete"
                  :disabled="deletingId === product.id"
                  @click="deleteProduct(product.id)"
                >
                  {{ deletingId === product.id ? 'Deleting...' : 'Delete' }}
                </button>
              </div>
              <div v-if="editingProductId === product.id" class="admin-inline-editor">
                <p v-if="loadingProductId === product.id" class="admin-helper-copy">Loading product detail...</p>
                <div class="admin-form-grid">
                  <label class="shop-field">
                    <span>Category</span>
                    <select v-model="productDrafts[product.id].category_slug">
                      <option value="">No category</option>
                      <option v-for="category in categoryOptions" :key="category.id" :value="category.slug">
                        {{ category.name }}
                      </option>
                    </select>
                  </label>
                  <label class="shop-field">
                    <span>Name</span>
                    <input v-model="productDrafts[product.id].name" type="text">
                  </label>
                  <label class="shop-field">
                    <span>Slug</span>
                    <input v-model="productDrafts[product.id].slug" type="text">
                  </label>
                  <label class="shop-field">
                    <span>Status</span>
                    <select v-model="productDrafts[product.id].status">
                      <option value="active">Active</option>
                      <option value="draft">Draft</option>
                      <option value="archived">Archived</option>
                    </select>
                  </label>
                  <label class="shop-field admin-checkbox">
                    <span>Featured</span>
                    <input v-model="productDrafts[product.id].is_featured" type="checkbox">
                  </label>
                  <label class="shop-field">
                    <span>Upload Images</span>
                    <input type="file" accept="image/*" @change="uploadImage($event, product.id)">
                  </label>
                  <label class="shop-field">
                    <span>Primary Image URL</span>
                    <input
                      v-model="productDrafts[product.id].primary_image_url"
                      type="text"
                      :placeholder="isUploadingImage && uploadingProductId === product.id ? 'Uploading to Supabase...' : 'https://...'"
                    >
                  </label>
                  <div v-if="productDrafts[product.id].gallery_image_urls.length" class="admin-field-span admin-gallery-shell">
                    <div class="admin-gallery-header">
                      <span>Gallery Preview</span>
                      <small>Choose the order and primary storefront image.</small>
                    </div>
                    <div class="admin-gallery-grid">
                      <article
                        v-for="imageUrl in productDrafts[product.id].gallery_image_urls"
                        :key="imageUrl"
                        class="admin-gallery-card"
                        :class="{ 'is-primary': productDrafts[product.id].primary_image_url === imageUrl }"
                      >
                        <div class="admin-gallery-media">
                          <img :src="imageUrl" alt="" loading="lazy">
                        </div>
                        <div class="admin-gallery-actions">
                          <button type="button" class="secondary-link admin-gallery-button" @click="setPrimaryImageForDraft(product.id, imageUrl)">
                            {{ productDrafts[product.id].primary_image_url === imageUrl ? 'Primary' : 'Make Primary' }}
                          </button>
                          <button type="button" class="secondary-link admin-gallery-button admin-gallery-remove" @click="removeGalleryImageFromDraft(product.id, imageUrl)">
                            Remove
                          </button>
                        </div>
                      </article>
                    </div>
                  </div>
                  <div class="admin-field-span admin-variants-shell">
                    <div class="admin-variants-header">
                      <div>
                        <span>Variants</span>
                        <small>Edit existing variants, add new ones, or remove options you no longer sell.</small>
                      </div>
                      <button type="button" class="secondary-link admin-variant-add" @click="addVariantDraft(product.id)">
                        Add Variant
                      </button>
                    </div>

                    <div class="admin-variant-list">
                      <article
                        v-for="(variant, index) in productDrafts[product.id].variants"
                        :key="variant.id || `${variant.sku}-${index}`"
                        class="admin-variant-card"
                      >
                        <div class="admin-variant-card-top">
                          <strong>Variant {{ index + 1 }}</strong>
                          <button type="button" class="secondary-link admin-variant-remove" @click="removeVariantDraft(product.id, index)">
                            Remove
                          </button>
                        </div>
                        <div class="admin-form-grid">
                          <label class="shop-field">
                            <span>Name</span>
                            <input v-model="variant.name" type="text">
                          </label>
                          <label class="shop-field">
                            <span>SKU</span>
                            <input v-model="variant.sku" type="text">
                          </label>
                          <label class="shop-field">
                            <span>Size</span>
                            <input v-model="variant.size" type="text">
                          </label>
                          <label class="shop-field">
                            <span>Color</span>
                            <input v-model="variant.color" type="text">
                          </label>
                          <label class="shop-field">
                            <span>Price</span>
                            <input v-model="variant.price" type="text">
                          </label>
                          <label class="shop-field">
                            <span>Compare At</span>
                            <input v-model="variant.compare_at_price" type="text">
                          </label>
                          <label class="shop-field">
                            <span>Stock</span>
                            <input v-model="variant.stock_quantity" type="number" min="0">
                          </label>
                          <label class="shop-field admin-checkbox">
                            <span>Active</span>
                            <input v-model="variant.is_active" type="checkbox" disabled>
                          </label>
                        </div>
                      </article>
                    </div>
                  </div>
                  <label class="shop-field admin-field-span">
                    <span>Short Description</span>
                    <textarea v-model="productDrafts[product.id].short_description" rows="3" />
                  </label>
                  <label class="shop-field admin-field-span">
                    <span>Description</span>
                    <textarea v-model="productDrafts[product.id].description" rows="4" />
                  </label>
                </div>
                <button type="button" class="hero-cta admin-submit" @click="saveProduct(product.id)">
                  Save Product
                </button>
              </div>
            </article>
          </div>
        </div>
      </div>
    </section>
  </AdminShell>
</template>
