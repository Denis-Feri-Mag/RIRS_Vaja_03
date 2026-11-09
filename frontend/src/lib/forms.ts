import type { FieldValues, Path, UseFormSetError } from 'react-hook-form'
import { ApiError } from './api'

/** Puts a backend validation error on the matching field, otherwise returns the message. */
export function applyServerError<T extends FieldValues>(err: unknown, setError: UseFormSetError<T>, fields: string[]): string | null {
  if (err instanceof ApiError) {
    if (err.field && fields.includes(err.field)) {
      setError(err.field as Path<T>, { message: err.message })
      return null
    }
    return err.message
  }
  return 'Prišlo je do napake.'
}
