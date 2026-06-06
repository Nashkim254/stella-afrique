<script setup lang="ts">
const config = useRuntimeConfig()
const { adminRole } = useAdminAuth()

type StaffRecord = {
  id: string
  email: string
  full_name: string
  role: string
  is_active: boolean
  last_login_at?: string | null
  created_at: string
}

const feedback = ref('')
const isCreating = ref(false)
const savingStaffId = ref<string | null>(null)
const newStaff = reactive({
  full_name: '',
  email: '',
  role: 'catalog',
  password: '',
})

const draftById = reactive<Record<string, {
  full_name: string
  role: string
  is_active: boolean
  password: string
}>>({})

const adminFetchOptions = {
  baseURL: config.public.apiBaseUrl,
  credentials: 'include' as const,
}

const extractErrorMessage = (error: any, fallback: string) =>
  error?.data?.error
  ?? error?.data?.message
  ?? error?.statusMessage
  ?? error?.message
  ?? fallback

const isOwner = computed(() => (adminRole.value ?? '').toLowerCase() === 'owner')

const { data: staff, refresh } = await useFetch<StaffRecord[]>(
  () => isOwner.value ? '/admin/staff' : null,
  {
    ...adminFetchOptions,
    default: () => [],
    server: false,
    watch: [isOwner],
  },
)

watch(
  staff,
  (value) => {
    for (const member of value ?? []) {
      draftById[member.id] = {
        full_name: member.full_name,
        role: member.role,
        is_active: member.is_active,
        password: '',
      }
    }
  },
  { immediate: true },
)

const createStaff = async () => {
  isCreating.value = true
  feedback.value = ''

  try {
    await $fetch('/admin/staff', {
      method: 'POST',
      ...adminFetchOptions,
      body: {
        full_name: newStaff.full_name,
        email: newStaff.email,
        role: newStaff.role,
        password: newStaff.password,
      },
    })

    feedback.value = 'Staff member created.'
    newStaff.full_name = ''
    newStaff.email = ''
    newStaff.role = 'catalog'
    newStaff.password = ''
    await refresh()
  }
  catch (error: any) {
    feedback.value = extractErrorMessage(error, 'Unable to create staff user.')
  }
  finally {
    isCreating.value = false
  }
}

const saveStaff = async (staffId: string) => {
  const draft = draftById[staffId]
  if (!draft) {
    return
  }

  savingStaffId.value = staffId
  feedback.value = ''

  try {
    await $fetch(`/admin/staff/${staffId}`, {
      method: 'PATCH',
      ...adminFetchOptions,
      body: {
        full_name: draft.full_name,
        role: draft.role,
        is_active: draft.is_active,
        password: draft.password.trim() || undefined,
      },
    })

    draft.password = ''
    feedback.value = 'Staff member updated.'
    await refresh()
  }
  catch (error: any) {
    feedback.value = extractErrorMessage(error, 'Unable to update staff user.')
  }
  finally {
    savingStaffId.value = null
  }
}
</script>

<template>
  <AdminShell
    title="Staff accounts and operational roles."
    description="Control who can access finance, catalogue, fulfilment, and other admin workflows."
  >
    <section class="admin-content-section">
      <p v-if="feedback" class="admin-feedback">{{ feedback }}</p>

      <div v-if="!isOwner" class="empty-state compact-empty">
        <h2>Owner access required.</h2>
        <p>Only the owner role can manage staff accounts and role assignments.</p>
      </div>

      <div v-else class="admin-grid">
        <div class="admin-panel">
          <div class="admin-section-heading">
            <div>
              <h2>Create Staff User</h2>
              <p>Add operations, catalogue, fulfilment, finance, or full admin access.</p>
            </div>
          </div>

          <div class="admin-form-grid">
            <label class="shop-field">
              <span>Full Name</span>
              <input v-model="newStaff.full_name" type="text">
            </label>
            <label class="shop-field">
              <span>Email</span>
              <input v-model="newStaff.email" type="email" autocomplete="username">
            </label>
            <label class="shop-field">
              <span>Role</span>
              <select v-model="newStaff.role">
                <option value="catalog">Catalog</option>
                <option value="fulfilment">Fulfilment</option>
                <option value="finance">Finance</option>
                <option value="admin">Admin</option>
                <option value="owner">Owner</option>
              </select>
            </label>
            <label class="shop-field">
              <span>Temporary Password</span>
              <input v-model="newStaff.password" type="password" autocomplete="new-password">
            </label>
          </div>

          <button type="button" class="hero-cta admin-submit" :disabled="isCreating" @click="createStaff">
            {{ isCreating ? 'Creating...' : 'Create Staff User' }}
          </button>
        </div>

        <div class="admin-panel">
          <div class="admin-section-heading">
            <div>
              <h2>Staff Directory</h2>
              <p>Update roles, activation state, and reset passwords as needed.</p>
            </div>
          </div>

          <div class="admin-table">
            <article
              v-for="member in staff"
              :key="member.id"
              class="admin-row admin-staff-row"
            >
              <div class="admin-staff-main">
                <strong>{{ member.full_name }}</strong>
                <p>{{ member.email }}</p>
                <p>
                  Added {{ new Date(member.created_at).toLocaleDateString() }}
                  <template v-if="member.last_login_at">
                    · Last login {{ new Date(member.last_login_at).toLocaleString() }}
                  </template>
                </p>
              </div>

              <div class="admin-staff-controls">
                <label class="shop-field">
                  <span>Role</span>
                  <select v-model="draftById[member.id].role">
                    <option value="catalog">Catalog</option>
                    <option value="fulfilment">Fulfilment</option>
                    <option value="finance">Finance</option>
                    <option value="admin">Admin</option>
                    <option value="owner">Owner</option>
                  </select>
                </label>
                <label class="shop-field">
                  <span>Full Name</span>
                  <input v-model="draftById[member.id].full_name" type="text">
                </label>
                <label class="shop-field">
                  <span>Reset Password</span>
                  <input v-model="draftById[member.id].password" type="password" placeholder="Leave blank to keep current">
                </label>
                <label class="shop-field admin-inline-check">
                  <span>Active</span>
                  <input v-model="draftById[member.id].is_active" type="checkbox">
                </label>
                <button
                  type="button"
                  class="hero-cta admin-submit"
                  :disabled="savingStaffId === member.id"
                  @click="saveStaff(member.id)"
                >
                  {{ savingStaffId === member.id ? 'Saving...' : 'Save Staff' }}
                </button>
              </div>
            </article>
          </div>
        </div>
      </div>
    </section>
  </AdminShell>
</template>
