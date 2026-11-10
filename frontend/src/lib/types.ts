export type Role = 'CUSTOMER' | 'PROVIDER' | 'ADMIN'
export type JobStatus = 'OPEN' | 'QUOTED' | 'ACCEPTED' | 'IN_PROGRESS' | 'COMPLETED' | 'REVIEWED' | 'CANCELLED' | 'EXPIRED'
export type QuoteStatus = 'PENDING' | 'ACCEPTED' | 'REJECTED' | 'WITHDRAWN'
export type VerificationStatus = 'PENDING' | 'VERIFIED' | 'REJECTED'

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
  active_quote_count: number
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

export interface MatchingItem {
  id: string
  title: string
  category_name: string
  city: string
  postal_code: string
  preferred_from: string | null
  preferred_to: string | null
  budget_min: number | null
  budget_max: number | null
  status: JobStatus
  quote_count: number
  quote_sent: boolean
  created_at: string
}

export interface MatchingPage {
  items: MatchingItem[]
  page: number
  page_size: number
  total: number
  verification_status: VerificationStatus
  category_names: string[]
  service_postal_codes: string[]
}

export interface Quote {
  id: string
  job_request_id: string
  provider_id: string
  provider_name: string
  provider_avg_rating: number
  provider_completed_jobs: number
  price: number
  estimated_hours: number
  earliest_start: string
  message: string | null
  status: QuoteStatus
  created_at: string
}

export interface MyQuoteItem {
  quote: Quote
  job_title: string
  job_status: JobStatus
  job_city: string
}

export interface ProviderProfile {
  id: string
  business_name: string
  tax_number: string
  description: string
  service_city: string
  service_postal_codes: string[]
  category_ids: string[]
  category_names: string[]
  verification_status: VerificationStatus
  avg_rating: number
  completed_jobs: number
}
