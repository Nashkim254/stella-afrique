import type { ProductRecord } from './useStorefrontContent'

export type CatalogProductCardResponse = {
  id: string
  slug: string
  name: string
  category?: string | null
  category_slug?: string | null
  short_description?: string | null
  price?: string | number | null
  primary_image_url?: string | null
}

export type CatalogProductsResponse = {
  products: CatalogProductCardResponse[]
}

export type CatalogProductDetailResponse = {
  id: string
  slug: string
  name: string
  category?: string | null
  category_slug?: string | null
  short_description?: string | null
  description?: string | null
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

export const useCatalogApi = () => {
  const toPriceString = (value?: string | number | null) => {
    if (value === null || value === undefined) {
      return ''
    }

    return String(value)
  }

  const mapCatalogCardToProductRecord = (
    product: CatalogProductCardResponse,
    fallback?: ProductRecord,
  ): ProductRecord => ({
    id: product.id ?? fallback?.id ?? crypto.randomUUID(),
    slug: product.slug,
    name: product.name,
    category: product.category ?? fallback?.category ?? 'Collection',
    categorySlug: product.category_slug ?? fallback?.categorySlug ?? 'new-in',
    price: toPriceString(product.price) || fallback?.price || '',
    image: product.primary_image_url ?? fallback?.image ?? '/images/products/fashion-01.jpg',
    shortDescription: product.short_description ?? fallback?.shortDescription ?? '',
    accent: fallback?.accent,
    story: fallback?.story ?? 'A curated storefront piece selected for daily dressing and soft occasion styling.',
    materials: fallback?.materials ?? ['Selected fabrication'],
    colors: fallback?.colors ?? ['Seasonal palette'],
    sizes: fallback?.sizes ?? ['S', 'M', 'L'],
    gallery: fallback?.gallery ?? [product.primary_image_url ?? fallback?.image ?? '/images/products/fashion-01.jpg'],
  })

  const mergeProductDetail = (
    detail: CatalogProductDetailResponse,
    fallback?: ProductRecord,
  ): ProductRecord => ({
    id: detail.id ?? fallback?.id ?? crypto.randomUUID(),
    slug: detail.slug,
    name: detail.name,
    category: detail.category ?? fallback?.category ?? 'Collection',
    categorySlug: detail.category_slug ?? fallback?.categorySlug ?? 'new-in',
    price: toPriceString(detail.variants[0]?.price) || fallback?.price || '',
    image: detail.images.find((image) => image.is_primary)?.image_url
      ?? detail.images[0]?.image_url
      ?? fallback?.image
      ?? '/images/products/fashion-01.jpg',
    shortDescription: detail.short_description ?? fallback?.shortDescription ?? '',
    accent: fallback?.accent,
    story: detail.description ?? fallback?.story ?? 'A curated storefront piece selected for daily dressing and soft occasion styling.',
    materials: fallback?.materials ?? ['Selected fabrication'],
    colors: [
      ...new Set([
        ...detail.variants.map((variant) => variant.color).filter(Boolean) as string[],
        ...(fallback?.colors ?? []),
      ]),
    ],
    sizes: [
      ...new Set([
        ...detail.variants.map((variant) => variant.size).filter(Boolean) as string[],
        ...(fallback?.sizes ?? []),
      ]),
    ],
    gallery: detail.images.length
      ? detail.images.map((image) => image.image_url)
      : (fallback?.gallery ?? [fallback?.image ?? '/images/products/fashion-01.jpg']),
  })

  return {
    mapCatalogCardToProductRecord,
    mergeProductDetail,
  }
}
