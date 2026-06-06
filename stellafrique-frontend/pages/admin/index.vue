<script setup lang="ts">
const config = useRuntimeConfig()
const { formatPrice } = useStorefrontContent()

type DashboardOverview = {
  gross_revenue: string | number
  paid_revenue: string | number
  average_order_value: string | number
  collection_rate_percentage: string | number
  total_orders: number
  orders_today: number
  paid_orders: number
  unpaid_orders: number
  pending_fulfilment_orders: number
  active_products: number
  low_stock_variants: number
}

type BreakdownPoint = {
  label: string
  count: number
}

type SalesSeriesPoint = {
  label: string
  orders: number
  gross_revenue: string | number
  paid_revenue: string | number
}

type TopProductPoint = {
  product_name: string
  units_sold: number
  revenue: string | number
}

type TopCategoryPoint = {
  category_name: string
  units_sold: number
  revenue: string | number
}

type PaymentMethodPoint = {
  label: string
  count: number
}

type InventoryAlertPoint = {
  variant_id: string
  product_name: string
  variant_name: string
  sku: string
  stock_quantity: number
}

type RecentOrderPoint = {
  order_number: string
  customer_name: string
  status: string
  payment_status: string
  total_amount: string | number
  created_at: string
}

type AdminDashboardResponse = {
  overview: DashboardOverview
  sales_series: SalesSeriesPoint[]
  fulfilment_breakdown: BreakdownPoint[]
  payment_breakdown: BreakdownPoint[]
  payment_method_breakdown: PaymentMethodPoint[]
  top_products: TopProductPoint[]
  top_categories: TopCategoryPoint[]
  inventory_alerts: InventoryAlertPoint[]
  recent_orders: RecentOrderPoint[]
}

const adminFetchOptions = {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include' as const,
}

const { data: dashboard } = await useFetch<AdminDashboardResponse>('/admin/dashboard', {
  ...adminFetchOptions,
  server: false,
  default: () => ({
    overview: {
      gross_revenue: '0',
      paid_revenue: '0',
      average_order_value: '0',
      collection_rate_percentage: '0',
      total_orders: 0,
      orders_today: 0,
      paid_orders: 0,
      unpaid_orders: 0,
      pending_fulfilment_orders: 0,
      active_products: 0,
      low_stock_variants: 0,
    },
    sales_series: [],
    fulfilment_breakdown: [],
    payment_breakdown: [],
    payment_method_breakdown: [],
    top_products: [],
    top_categories: [],
    inventory_alerts: [],
    recent_orders: [],
  }),
})

const kpiCards = computed(() => [
  {
    label: 'Gross Revenue',
    value: formatPrice(String(dashboard.value.overview.gross_revenue)),
    accent: 'pink',
  },
  {
    label: 'Collected Revenue',
    value: formatPrice(String(dashboard.value.overview.paid_revenue)),
    accent: 'violet',
  },
  {
    label: 'Collection Rate',
    value: `${Number(dashboard.value.overview.collection_rate_percentage).toFixed(1)}%`,
    accent: 'mint',
  },
  {
    label: 'Average Order',
    value: formatPrice(String(dashboard.value.overview.average_order_value)),
    accent: 'sky',
  },
  {
    label: 'Orders Today',
    value: String(dashboard.value.overview.orders_today),
    accent: 'mint',
  },
  {
    label: 'Pending Fulfilment',
    value: String(dashboard.value.overview.pending_fulfilment_orders),
    accent: 'violet',
  },
  {
    label: 'Unpaid Orders',
    value: String(dashboard.value.overview.unpaid_orders),
    accent: 'sky',
  },
  {
    label: 'Low-Stock Variants',
    value: String(dashboard.value.overview.low_stock_variants),
    accent: 'pink',
  },
])

const salesChart = computed(() => {
  const peak = Math.max(...dashboard.value.sales_series.map((point) => Number(point.orders)), 1)
  return dashboard.value.sales_series.map((point) => ({
    ...point,
    value: Number(point.orders),
    ratio: peak ? Number(point.orders) / peak : 0,
  }))
})

const lineChartPoints = computed(() => {
  const points = salesChart.value
  if (!points.length) {
    return ''
  }

  const width = 1000
  const height = 260
  const leftPadding = 40
  const rightPadding = 28
  const topPadding = 18
  const bottomPadding = 26
  const chartWidth = width - leftPadding - rightPadding
  const chartHeight = height - topPadding - bottomPadding
  const step = points.length > 1 ? chartWidth / (points.length - 1) : 0

  return points
    .map((point, index) => {
      const x = leftPadding + step * index
      const y = topPadding + chartHeight - point.ratio * chartHeight
      return `${x},${y}`
    })
    .join(' ')
})

const lineChartArea = computed(() => {
  if (!lineChartPoints.value) {
    return ''
  }

  const width = 1000
  const height = 260
  const leftPadding = 40
  const bottomPadding = 26
  const baseY = height - bottomPadding
  return `${leftPadding},${baseY} ${lineChartPoints.value} ${width - 28},${baseY}`
})
</script>

<template>
  <AdminShell
    title="Business command center."
    description="Track revenue, payment collection, fulfilment pressure, and stock risk before they become operational problems."
  >
    <section class="admin-dashboard-grid">
      <div class="admin-kpi-grid">
        <article
          v-for="card in kpiCards"
          :key="card.label"
          class="admin-kpi-card"
          :class="`is-${card.accent}`"
        >
          <span>{{ card.label }}</span>
          <strong>{{ card.value }}</strong>
        </article>
      </div>

      <div class="admin-dashboard-panel admin-dashboard-chart">
        <div class="admin-section-heading">
          <div>
            <h2>14-Day Order Trend</h2>
            <p>Continuous order flow over the last two weeks, with revenue context kept secondary.</p>
          </div>
        </div>
        <div class="admin-line-chart-shell">
          <svg class="admin-line-chart" viewBox="0 0 1000 260" preserveAspectRatio="none" aria-hidden="true">
            <defs>
              <linearGradient id="adminLineArea" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stop-color="rgba(251, 46, 134, 0.24)" />
                <stop offset="100%" stop-color="rgba(251, 46, 134, 0.02)" />
              </linearGradient>
              <linearGradient id="adminLineStroke" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stop-color="#fb2e86" />
                <stop offset="100%" stop-color="#7e33e0" />
              </linearGradient>
            </defs>
            <line
              v-for="gridLine in 5"
              :key="gridLine"
              x1="40"
              :y1="18 + ((gridLine - 1) * (216 / 4))"
              x2="972"
              :y2="18 + ((gridLine - 1) * (216 / 4))"
              class="admin-line-chart-grid"
            />
            <polygon v-if="lineChartArea" :points="lineChartArea" class="admin-line-chart-area" />
            <polyline v-if="lineChartPoints" :points="lineChartPoints" class="admin-line-chart-stroke" />
            <circle
              v-for="point in salesChart"
              :key="point.label"
              :cx="40 + ((salesChart.length > 1 ? 932 / (salesChart.length - 1) : 0) * salesChart.indexOf(point))"
              :cy="18 + 216 - (point.ratio * 216)"
              r="6"
              class="admin-line-chart-point"
            />
          </svg>
          <div class="admin-line-chart-labels">
            <article v-for="point in salesChart" :key="point.label" class="admin-line-chart-label">
              <strong>{{ point.orders }}</strong>
              <span>{{ point.label }}</span>
              <small>{{ formatPrice(String(point.paid_revenue)) }}</small>
            </article>
          </div>
        </div>
      </div>

      <div class="admin-dashboard-lower-grid">
        <div class="admin-dashboard-panel admin-dashboard-table">
          <div class="admin-section-heading">
            <div>
              <h2>Recent Orders</h2>
              <p>Latest customer orders with payment and fulfilment states.</p>
            </div>
            <NuxtLink to="/admin/orders" class="secondary-link">Manage Orders</NuxtLink>
          </div>
          <div class="admin-table">
            <article v-for="order in dashboard.recent_orders" :key="order.order_number" class="admin-row admin-order-row">
              <div>
                <strong>{{ order.order_number }}</strong>
                <p>{{ order.customer_name }}</p>
              </div>
              <div class="admin-order-meta">
                <span class="admin-status-pill">{{ order.status }}</span>
                <span class="admin-status-pill admin-status-pill-secondary">{{ order.payment_status }}</span>
                <strong>{{ formatPrice(String(order.total_amount)) }}</strong>
              </div>
            </article>
          </div>
        </div>

        <div class="admin-dashboard-panel admin-dashboard-side admin-dashboard-alerts">
          <div class="admin-section-heading">
            <div>
              <h2>Inventory Alerts</h2>
              <p>Active variants nearing stock-out.</p>
            </div>
            <NuxtLink to="/admin/inventory" class="secondary-link">Open Inventory</NuxtLink>
          </div>
          <div class="admin-performance-list">
            <article v-for="item in dashboard.inventory_alerts" :key="item.variant_id" class="admin-performance-row admin-alert-row">
              <div>
                <strong>{{ item.product_name }}</strong>
                <p>{{ item.variant_name }} · {{ item.sku }}</p>
              </div>
              <strong>{{ item.stock_quantity }} left</strong>
            </article>
            <p v-if="!dashboard.inventory_alerts.length" class="route-copy">No critical stock alerts right now.</p>
          </div>
        </div>

        <div class="admin-dashboard-panel admin-dashboard-side admin-dashboard-products">
          <div class="admin-section-heading">
            <div>
              <h2>Top Products</h2>
              <p>Best performers by units sold across live orders.</p>
            </div>
          </div>
          <div class="admin-performance-list">
            <article v-for="item in dashboard.top_products" :key="item.product_name" class="admin-performance-row">
              <div>
                <strong>{{ item.product_name }}</strong>
                <p>{{ item.units_sold }} units sold</p>
              </div>
              <strong>{{ formatPrice(String(item.revenue)) }}</strong>
            </article>
          </div>
        </div>

        <div class="admin-dashboard-panel admin-dashboard-side admin-dashboard-products">
          <div class="admin-section-heading">
            <div>
              <h2>Top Categories</h2>
              <p>Where sales concentration is building across the assortment.</p>
            </div>
          </div>
          <div class="admin-performance-list">
            <article v-for="item in dashboard.top_categories" :key="item.category_name" class="admin-performance-row">
              <div>
                <strong>{{ item.category_name }}</strong>
                <p>{{ item.units_sold }} units sold</p>
              </div>
              <strong>{{ formatPrice(String(item.revenue)) }}</strong>
            </article>
          </div>
        </div>

        <div class="admin-dashboard-panel admin-dashboard-mini">
          <div class="admin-section-heading">
            <div>
              <h2>Fulfilment Mix</h2>
              <p>Operational load by order status.</p>
            </div>
          </div>
          <div class="admin-breakdown-list">
            <article v-for="item in dashboard.fulfilment_breakdown" :key="item.label" class="admin-breakdown-row">
              <span>{{ item.label }}</span>
              <strong>{{ item.count }}</strong>
            </article>
          </div>
        </div>

        <div class="admin-dashboard-panel admin-dashboard-mini">
          <div class="admin-section-heading">
            <div>
              <h2>Payment Mix</h2>
              <p>How many orders are collected versus still open.</p>
            </div>
          </div>
          <div class="admin-breakdown-list">
            <article v-for="item in dashboard.payment_breakdown" :key="item.label" class="admin-breakdown-row">
              <span>{{ item.label }}</span>
              <strong>{{ item.count }}</strong>
            </article>
          </div>
        </div>

        <div class="admin-dashboard-panel admin-dashboard-mini">
          <div class="admin-section-heading">
            <div>
              <h2>Payment Rails</h2>
              <p>Which payment methods are carrying current order volume.</p>
            </div>
          </div>
          <div class="admin-breakdown-list">
            <article v-for="item in dashboard.payment_method_breakdown" :key="item.label" class="admin-breakdown-row">
              <span>{{ item.label }}</span>
              <strong>{{ item.count }}</strong>
            </article>
          </div>
        </div>
      </div>
    </section>
  </AdminShell>
</template>
