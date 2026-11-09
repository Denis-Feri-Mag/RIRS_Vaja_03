export type Role = 'CUSTOMER' | 'PROVIDER' | 'ADMIN'
export type JobStatus = 'OPEN' | 'QUOTED' | 'ACCEPTED' | 'IN_PROGRESS' | 'COMPLETED' | 'REVIEWED' | 'CANCELLED' | 'EXPIRED'

export interface User {
  id: string
  email: string
  full_name: string
  phone: string | null
  role: Role
}

export interface Category {
  id: string
  name: string
  slug: string
}

export interface Job {
  id: string
  category_id: string
  category_name: string
  title: string
  description: string
  city: string
  postal_code: string
  address_line: string | null
  preferred_from: string | null
  preferred_to: string | null
  budget_min: number | null
  budget_max: number | null
  status: JobStatus
  expires_at: string
  created_at: string
  is_owner: boolean
}

export interface JobListItem {
  id: string
  title: string
  category_name: string
  city: string
  status: JobStatus
  quote_count: number
  created_at: string
}
