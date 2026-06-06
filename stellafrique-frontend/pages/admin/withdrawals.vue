<script setup lang="ts">
const config = useRuntimeConfig()
const { formatPrice } = useStorefrontContent()
const { adminRole } = useAdminAuth()

type ReleasableOrder = {
  currency: string
  collected_revenue: string | number
  successful_withdrawals: string | number
  pending_withdrawals: string | number
  available_balance: string | number
}

type WithdrawalRecord = {
  id: string
  release_reference: string
  amount: string | number
  currency: string
  destination_type: string
  destination: string
  status: string
  requested_by_email: string
  requested_by_role: string
  payout_id?: string | null
  receipt_number?: string | null
  external_request_id?: string | null
  failure_reason?: string | null
  created_at: string
  updated_at: string
  completed_at?: string | null
}

type WithdrawalsOverview = {
  balance: ReleasableOrder
  withdrawals: WithdrawalRecord[]
}

const feedback = ref('')
const releaseDestination = ref('')
const releaseAmount = ref('')
const isSubmitting = ref(false)

const adminFetchOptions = {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include' as const,
}

const canManageWithdrawals = computed(() =>
  ['owner', 'admin'].includes((adminRole.value ?? '').toLowerCase()),
)

const extractErrorMessage = (error: any, fallback: string) =>
  error?.data?.error
  ?? error?.data?.message
  ?? error?.statusMessage
  ?? error?.message
  ?? fallback

const { data: overview, refresh } = await useFetch<WithdrawalsOverview>(
  () => canManageWithdrawals.value ? '/admin/withdrawals' : null,
  {
    ...adminFetchOptions,
    server: false,
    default: () => ({
      balance: {
        currency: 'KES',
        collected_revenue: 0,
        successful_withdrawals: 0,
        pending_withdrawals: 0,
        available_balance: 0,
      },
      withdrawals: [],
    }),
    watch: [canManageWithdrawals],
  },
)

watch(
  () => overview.value?.balance,
  (balance) => {
    if (balance && releaseAmount.value === '') {
      releaseAmount.value = String(balance.available_balance ?? '')
    }
  },
  { immediate: true },
)

const submitRelease = async () => {
  isSubmitting.value = true
  feedback.value = ''
  const normalizedAmount = String(releaseAmount.value ?? '').trim()

  try {
    const result = await $fetch<{
      status: string
      release_reference: string
      message: string
    }>('/admin/withdrawals', {
      method: 'POST',
      ...adminFetchOptions,
      body: {
        destination_type: 'phone',
        destination: releaseDestination.value,
        amount: normalizedAmount === ''
          ? undefined
          : Number(normalizedAmount),
      },
    })

    feedback.value = `${result.message} Release reference: ${result.release_reference}.`
    releaseAmount.value = ''
    await refresh()
  }
  catch (error: any) {
    feedback.value = extractErrorMessage(error, 'Unable to release funds.')
  }
  finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <AdminShell
    title="Business withdrawals and payout history."
    description="Manage merchant cash-out requests separately from order handling, with a clear history of release attempts and outcomes."
  >
    <section class="admin-content-section">
      <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>

      <div v-if="!canManageWithdrawals" class="empty-state compact-empty">
        <h2>Admin access required.</h2>
        <p>Only staff with owner or admin roles can manage Velipay business withdrawals.</p>
      </div>

      <div v-else class="admin-grid">
        <div class="admin-panel">
          <div class="admin-section-heading">
            <div>
              <h2>Available Balance</h2>
              <p>Derived from successful customer payments minus completed and pending withdrawals in your own system.</p>
            </div>
          </div>

          <div class="admin-order-summary-grid">
            <div>
              <span>Collected Revenue</span>
              <strong>{{ formatPrice(String(overview.balance.collected_revenue)) }}</strong>
            </div>
            <div>
              <span>Successful Withdrawals</span>
              <strong>{{ formatPrice(String(overview.balance.successful_withdrawals)) }}</strong>
            </div>
            <div>
              <span>Pending Withdrawals</span>
              <strong>{{ formatPrice(String(overview.balance.pending_withdrawals)) }}</strong>
            </div>
            <div>
              <span>Available Balance</span>
              <strong>{{ formatPrice(String(overview.balance.available_balance)) }}</strong>
            </div>
          </div>

          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Destination Phone</span>
              <input
                v-model="releaseDestination"
                type="text"
                placeholder="2547XXXXXXXX"
              >
            </label>
            <label class="shop-field">
              <span>Withdrawal Amount (KES)</span>
              <input
                v-model="releaseAmount"
                type="number"
                min="0"
                step="1"
              >
            </label>
          </div>

          <button
            type="button"
            class="hero-cta admin-submit"
            :disabled="isSubmitting"
            @click="submitRelease"
          >
            {{ isSubmitting ? 'Releasing...' : 'Withdraw From Balance' }}
          </button>
        </div>

        <div class="admin-panel">
          <div class="admin-section-heading">
            <div>
              <h2>Withdrawal History</h2>
              <p>Track requested, successful, and failed cash-out events over time.</p>
            </div>
          </div>

          <div v-if="overview.withdrawals.length" class="admin-event-list">
            <article
              v-for="event in overview.withdrawals"
              :key="event.id"
              class="admin-event-card"
            >
              <div class="admin-event-meta">
                <span class="admin-status-pill">{{ event.status.replaceAll('_', ' ') }}</span>
                <strong>{{ new Date(event.created_at).toLocaleString() }}</strong>
              </div>
              <p class="admin-event-message">
                <strong>{{ formatPrice(String(event.amount)) }}</strong> · {{ event.destination }}
              </p>
              <p class="admin-event-actor">Requested by {{ event.requested_by_email }} · {{ event.requested_by_role }}</p>
              <p class="admin-event-details">Release reference: {{ event.release_reference }}</p>
              <p v-if="event.receipt_number" class="admin-event-details">Receipt: {{ event.receipt_number }}</p>
              <p v-if="event.external_request_id" class="admin-event-details">External request: {{ event.external_request_id }}</p>
              <p v-if="event.failure_reason" class="admin-event-details">Failure: {{ event.failure_reason }}</p>
              <p v-if="event.completed_at" class="admin-event-details">Completed: {{ new Date(event.completed_at).toLocaleString() }}</p>
            </article>
          </div>

          <div v-else class="empty-state compact-empty">
            <h2>No withdrawals yet.</h2>
            <p>Release requests and webhook outcomes will appear here once finance actions begin.</p>
          </div>
        </div>
      </div>
    </section>
  </AdminShell>
</template>
