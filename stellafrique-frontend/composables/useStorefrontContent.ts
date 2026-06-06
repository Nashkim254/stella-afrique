export type FeaturedProduct = {
  id: string
  category_id?: string | null
  name: string
  slug: string
  category: string
  short_description?: string | null
  price?: string | null
  primary_image_url?: string | null
}

export type ProductCard = {
  id: string
  slug: string
  name: string
  category: string
  categorySlug: string
  price: string
  image: string
  accent?: string
}

export type CategoryCard = {
  name: string
  slug: string
  count: string
  image: string
}

export type BlogCard = {
  slug: string
  title: string
  excerpt: string
  image: string
}

export type ProductRecord = {
  id: string
  slug: string
  name: string
  category: string
  categorySlug: string
  price: string
  image: string
  shortDescription: string
  accent?: string
  story: string
  materials: string[]
  colors: string[]
  sizes: string[]
  gallery: string[]
}

export const useStorefrontContent = () => {
  const storeName = 'Stellafrique'
  const currency = 'KES'

  const navLinks = [
    { label: 'New In', to: '/collections/new-in' },
    { label: 'Dresses', to: '/collections/dresses' },
    { label: 'Knitwear', to: '/collections/knitwear' },
    { label: 'Accessories', to: '/collections/accessories' },
    { label: 'Sale', to: '/collections/sale' },
  ]

  const latestTabs = ['New Arrival', 'Best Seller', 'Featured', 'Special Offer']

  const fallbackFeaturedProducts: FeaturedProduct[] = [
    {
      id: 'sample-1',
      name: 'Tailored Linen Set',
      slug: 'tailored-linen-set',
      category: 'Matching Set',
      short_description: 'Lightweight two-piece look built for warm city days.',
      price: '5400.00',
      primary_image_url: '/images/products/fashion-01.jpg',
    },
    {
      id: 'sample-2',
      name: 'Structured Weekend Dress',
      slug: 'structured-weekend-dress',
      category: 'Dresses',
      short_description: 'Clean silhouette with a soft drape and practical pockets.',
      price: '6200.00',
      primary_image_url: '/images/products/fashion-02.jpg',
    },
    {
      id: 'sample-3',
      name: 'Soft Neutral Knit',
      slug: 'soft-neutral-knit',
      category: 'Knitwear',
      short_description: 'Layer-friendly knit with an easy relaxed fit.',
      price: '4600.00',
      primary_image_url: '/images/products/fashion-06.jpg',
    },
    {
      id: 'sample-4',
      name: 'Everyday Occasion Blazer',
      slug: 'everyday-occasion-blazer',
      category: 'Outerwear',
      short_description: 'Clean tailoring for work, dinner, and weekends.',
      price: '7800.00',
      primary_image_url: '/images/products/fashion-15.jpg',
    },
  ]

  const latestProducts: ProductCard[] = [
    {
      id: 'latest-1',
      slug: 'cloudline-cardigan',
      name: 'Cloudline Cardigan',
      category: 'Knitwear',
      categorySlug: 'knitwear',
      price: '4900',
      image: '/images/products/fashion-17.jpg',
      accent: 'sky',
    },
    {
      id: 'latest-2',
      slug: 'rose-edit-blouse',
      name: 'Rose Edit Blouse',
      category: 'Tops',
      categorySlug: 'tops',
      price: '4300',
      image: '/images/products/fashion-09.jpg',
      accent: 'pink',
    },
    {
      id: 'latest-3',
      slug: 'soft-studio-layer',
      name: 'Soft Studio Layer',
      category: 'Outerwear',
      categorySlug: 'outerwear',
      price: '6900',
      image: '/images/products/fashion-07.jpg',
      accent: 'mint',
    },
    {
      id: 'latest-4',
      slug: 'weekend-shift-dress',
      name: 'Weekend Shift Dress',
      category: 'Dresses',
      categorySlug: 'dresses',
      price: '6200',
      image: '/images/products/fashion-18.jpg',
      accent: 'violet',
    },
    {
      id: 'latest-5',
      slug: 'minimal-day-shirt',
      name: 'Minimal Day Shirt',
      category: 'Shirts',
      categorySlug: 'shirts',
      price: '4500',
      image: '/images/products/fashion-10.jpg',
      accent: 'pink',
    },
    {
      id: 'latest-6',
      slug: 'signature-lounge-knit',
      name: 'Signature Lounge Knit',
      category: 'Basics',
      categorySlug: 'basics',
      price: '3800',
      image: '/images/products/fashion-12.jpg',
      accent: 'sky',
    },
  ]

  const offerCards = [
    {
      title: 'Free Delivery',
      copy: 'On Nairobi orders above KES 5,000 and easy support for every shipment.',
    },
    {
      title: 'Curated Looks',
      copy: 'Collections styled around everyday wear, soft tailoring, and occasion pieces.',
    },
    {
      title: 'Easy Exchange',
      copy: 'Simple size swaps within our exchange window for eligible full-price orders.',
    },
    {
      title: 'Gift Ready',
      copy: 'Clean packaging and elevated presentation for thoughtful gifting moments.',
    },
  ]

  const trendProducts: ProductCard[] = [
    {
      id: 'trend-1',
      slug: 'pastel-weekend-set',
      name: 'Pastel Weekend Set',
      category: 'Matching Set',
      categorySlug: 'new-in',
      price: '7100',
      image: '/images/products/fashion-03.jpg',
    },
    {
      id: 'trend-2',
      slug: 'soft-form-blazer',
      name: 'Soft Form Blazer',
      category: 'Outerwear',
      categorySlug: 'outerwear',
      price: '8200',
      image: '/images/products/fashion-04.jpg',
    },
    {
      id: 'trend-3',
      slug: 'daylight-knit',
      name: 'Daylight Knit',
      category: 'Knitwear',
      categorySlug: 'knitwear',
      price: '4700',
      image: '/images/products/fashion-19.jpg',
    },
    {
      id: 'trend-4',
      slug: 'rose-layer-shirt',
      name: 'Rose Layer Shirt',
      category: 'Tops',
      categorySlug: 'tops',
      price: '3900',
      image: '/images/products/fashion-20.jpg',
    },
  ]

  const categoryCards: CategoryCard[] = [
    {
      name: 'Knitwear',
      slug: 'knitwear',
      count: '24 pieces',
      image: '/images/products/fashion-06.jpg',
    },
    {
      name: 'Dresses',
      slug: 'dresses',
      count: '18 pieces',
      image: '/images/products/fashion-05.jpg',
    },
    {
      name: 'Accessories',
      slug: 'accessories',
      count: '12 pieces',
      image: '/images/products/fashion-16.jpg',
    },
    {
      name: 'Lounge Sets',
      slug: 'new-in',
      count: '16 pieces',
      image: '/images/products/fashion-14.jpg',
    },
  ]

  const blogCards: BlogCard[] = [
    {
      slug: 'style-soft-neutrals',
      title: 'How to style soft neutrals for every day',
      excerpt: 'Three ways to build outfits that stay polished without feeling overworked.',
      image: '/images/products/fashion-21.jpg',
    },
    {
      slug: 'build-better-layers',
      title: 'Building a small wardrobe with better layers',
      excerpt: 'A tighter edit of knits, shirts, and dresses that keeps styling simple.',
      image: '/images/products/fashion-10.jpg',
    },
    {
      slug: 'gifting-and-occasion-edits',
      title: 'From gifting picks to occasion edits',
      excerpt: 'Accessories, statement layers, and easy finishing pieces for thoughtful shopping.',
      image: '/images/products/fashion-13.jpg',
    },
  ]

  const brandLogos = [
    '/images/brands/brand-01.svg',
    '/images/brands/brand-02.svg',
    '/images/brands/brand-03.svg',
  ]

  const footerGroups = [
    {
      title: 'Categories',
      links: [
        { label: 'Dresses', to: '/collections/dresses' },
        { label: 'Knitwear', to: '/collections/knitwear' },
        { label: 'Accessories', to: '/collections/accessories' },
      ],
    },
    {
      title: 'Customer Care',
      links: [
        { label: 'Shop', to: '/shop' },
        { label: 'Sale', to: '/collections/sale' },
        { label: 'Contact', to: '/contact' },
      ],
    },
    {
      title: 'Pages',
      links: [
        { label: 'Blog', to: '/blog' },
        { label: 'Shop', to: '/shop' },
        { label: 'Contact', to: '/contact' },
      ],
    },
  ]

  const productDetails: Record<string, Omit<ProductRecord, 'id' | 'slug' | 'name' | 'category' | 'categorySlug' | 'price' | 'image' | 'shortDescription' | 'accent'>> = {
    'tailored-linen-set': {
      story: 'A clean matching set cut for movement, warm afternoons, and polished weekday dressing.',
      materials: ['Linen blend', 'Soft lining'],
      colors: ['Sand', 'Clay'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-01.jpg', '/images/products/fashion-10.jpg', '/images/products/fashion-15.jpg'],
    },
    'structured-weekend-dress': {
      story: 'A soft occasion dress with enough structure to carry from lunch dates into evening plans.',
      materials: ['Cotton sateen', 'Breathable lining'],
      colors: ['Cream', 'Rose'],
      sizes: ['S', 'M', 'L', 'XL'],
      gallery: ['/images/products/fashion-02.jpg', '/images/products/fashion-18.jpg', '/images/products/fashion-20.jpg'],
    },
    'soft-neutral-knit': {
      story: 'The everyday knit layer that works with denim, tailoring, and travel packing.',
      materials: ['Wool blend', 'Brushed finish'],
      colors: ['Oat', 'Stone'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-06.jpg', '/images/products/fashion-12.jpg', '/images/products/fashion-19.jpg'],
    },
    'everyday-occasion-blazer': {
      story: 'Relaxed tailoring with enough shape for work looks, celebrations, and smart gifting.',
      materials: ['Suiting twill', 'Smooth lining'],
      colors: ['Cream', 'Latte'],
      sizes: ['M', 'L', 'XL'],
      gallery: ['/images/products/fashion-15.jpg', '/images/products/fashion-11.jpg', '/images/products/fashion-16.jpg'],
    },
    'cloudline-cardigan': {
      story: 'A soft cardigan made for layering over dresses and simple tanks without adding bulk.',
      materials: ['Fine knit', 'Cotton blend'],
      colors: ['Blush', 'Cloud'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-17.jpg', '/images/products/fashion-12.jpg', '/images/products/fashion-06.jpg'],
    },
    'rose-edit-blouse': {
      story: 'A feminine blouse with enough structure to keep tailoring, skirts, and denim feeling intentional.',
      materials: ['Crisp cotton', 'Light drape finish'],
      colors: ['Rose', 'Pearl'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-09.jpg', '/images/products/fashion-13.jpg', '/images/products/fashion-20.jpg'],
    },
    'soft-studio-layer': {
      story: 'An easy outer layer designed for neat, minimal outfits and polished everyday errands.',
      materials: ['Structured cotton', 'Soft inner facing'],
      colors: ['Stone', 'Buttercream'],
      sizes: ['M', 'L', 'XL'],
      gallery: ['/images/products/fashion-07.jpg', '/images/products/fashion-18.jpg', '/images/products/fashion-02.jpg'],
    },
    'weekend-shift-dress': {
      story: 'A simple dress shape that feels dressed without needing much styling effort.',
      materials: ['Fluid crepe', 'Comfort stretch'],
      colors: ['Petal', 'Ivory'],
      sizes: ['S', 'M', 'L', 'XL'],
      gallery: ['/images/products/fashion-18.jpg', '/images/products/fashion-05.jpg', '/images/products/fashion-03.jpg'],
    },
    'minimal-day-shirt': {
      story: 'A clean shirt for office mornings, weekend markets, and soft layering under knitwear.',
      materials: ['Cotton poplin'],
      colors: ['White', 'Taupe'],
      sizes: ['S', 'M', 'L', 'XL'],
      gallery: ['/images/products/fashion-10.jpg', '/images/products/fashion-07.jpg', '/images/products/fashion-01.jpg'],
    },
    'signature-lounge-knit': {
      story: 'The off-duty knit set that keeps comfort high without losing shape or softness.',
      materials: ['Cotton knit', 'Brushed yarn'],
      colors: ['Cream', 'Sky'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-12.jpg', '/images/products/fashion-19.jpg', '/images/products/fashion-17.jpg'],
    },
    'pastel-weekend-set': {
      story: 'A playful set for weekend travel, gifting moments, and easy daytime styling.',
      materials: ['Soft cotton blend'],
      colors: ['Pink', 'Mint'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-03.jpg', '/images/products/fashion-18.jpg', '/images/products/fashion-13.jpg'],
    },
    'soft-form-blazer': {
      story: 'Relaxed tailoring with enough drape to work over dresses, denim, and knit separates.',
      materials: ['Tailored twill', 'Satin lining'],
      colors: ['Sand', 'Bone'],
      sizes: ['M', 'L', 'XL'],
      gallery: ['/images/products/fashion-04.jpg', '/images/products/fashion-15.jpg', '/images/products/fashion-16.jpg'],
    },
    'daylight-knit': {
      story: 'A softer knit option made for layering through cool mornings and rainy afternoons.',
      materials: ['Merino blend', 'Textured knit'],
      colors: ['Oat', 'Camel'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-19.jpg', '/images/products/fashion-06.jpg', '/images/products/fashion-12.jpg'],
    },
    'rose-layer-shirt': {
      story: 'A light shirt with a subtle statement colour for brightening neutral wardrobes.',
      materials: ['Washed cotton'],
      colors: ['Rose', 'Clay'],
      sizes: ['S', 'M', 'L'],
      gallery: ['/images/products/fashion-20.jpg', '/images/products/fashion-09.jpg', '/images/products/fashion-10.jpg'],
    },
  }

  const allProducts: ProductRecord[] = [
    ...fallbackFeaturedProducts.map((item) => ({
      id: item.id,
      slug: item.slug,
      name: item.name,
      category: item.category,
      categorySlug: item.category.toLowerCase().replace(/\s+/g, '-'),
      price: item.price ?? '',
      image: item.primary_image_url ?? '',
      shortDescription: item.short_description ?? '',
      ...productDetails[item.slug],
    })),
    ...latestProducts.map((item) => ({
      id: item.id,
      slug: item.slug,
      name: item.name,
      category: item.category,
      categorySlug: item.categorySlug,
      price: item.price,
      image: item.image,
      shortDescription: `${item.category} piece chosen for a refined, wearable wardrobe.`,
      accent: item.accent,
      ...productDetails[item.slug],
    })),
    ...trendProducts.map((item) => ({
      id: item.id,
      slug: item.slug,
      name: item.name,
      category: item.category,
      categorySlug: item.categorySlug,
      price: item.price,
      image: item.image,
      shortDescription: `${item.category} look with elevated styling potential.`,
      ...productDetails[item.slug],
    })),
  ].filter((item, index, arr) => arr.findIndex((candidate) => candidate.slug === item.slug) === index)

  const collectionOptions = Array.from(
    new Map(allProducts.map((item) => [item.categorySlug, { label: item.category, value: item.categorySlug }])).values(),
  )

  const formatPrice = (value?: string | null) => {
    if (!value) {
      return 'Price on request'
    }

    return new Intl.NumberFormat('en-KE', {
      style: 'currency',
      currency,
      maximumFractionDigits: 0,
    }).format(Number(value))
  }

  return {
    storeName,
    currency,
    navLinks,
    latestTabs,
    fallbackFeaturedProducts,
    latestProducts,
    offerCards,
    trendProducts,
    categoryCards,
    blogCards,
    brandLogos,
    footerGroups,
    allProducts,
    collectionOptions,
    formatPrice,
  }
}
