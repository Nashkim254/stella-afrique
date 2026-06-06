export type CartItem = {
  lineId: string
  slug: string
  name: string
  category: string
  image: string
  price: string
  variantId?: string
  variantName?: string
  sku?: string
  size?: string
  color?: string
  quantity: number
}

export const useCart = () => {
  const items = useState<CartItem[]>('cart-items', () => [])

  const addItem = (item: Omit<CartItem, 'quantity'>, quantity = 1) => {
    const existing = items.value.find((entry) => entry.lineId === item.lineId)

    if (existing) {
      existing.quantity += Math.max(1, quantity)
      return
    }

    items.value.push({
      ...item,
      quantity: Math.max(1, quantity),
    })
  }

  const removeItem = (lineId: string) => {
    items.value = items.value.filter((item) => item.lineId !== lineId)
  }

  const updateQuantity = (lineId: string, quantity: number) => {
    const item = items.value.find((entry) => entry.lineId === lineId)

    if (!item) {
      return
    }

    if (quantity <= 0) {
      removeItem(lineId)
      return
    }

    item.quantity = quantity
  }

  const clear = () => {
    items.value = []
  }

  const itemCount = computed(() =>
    items.value.reduce((count, item) => count + item.quantity, 0),
  )

  const subtotal = computed(() =>
    items.value.reduce((sum, item) => sum + (Number(item.price) * item.quantity), 0),
  )

  return {
    items,
    addItem,
    removeItem,
    updateQuantity,
    clear,
    itemCount,
    subtotal,
  }
}
