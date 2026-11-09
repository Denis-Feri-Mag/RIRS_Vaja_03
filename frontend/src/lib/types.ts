export type Role = 'CUSTOMER' | 'PROVIDER' | 'ADMIN'

export interface User {
  id: string
  email: string
  full_name: string
  phone: string | null
  role: Role
}
