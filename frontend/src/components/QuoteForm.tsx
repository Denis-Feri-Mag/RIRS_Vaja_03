import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { api } from '../lib/api'
import { applyServerError } from '../lib/forms'
import { today } from '../lib/format'
import type { Quote } from '../lib/types'

export const quoteSchema = z.object({
  price: z.coerce.number({ message: 'Vpišite ceno.' }).gt(0, 'Cena mora biti večja od 0.').max(100000, 'Cena je lahko največ 100.000 €.'),
  estimated_hours: z.coerce.number({ message: 'Vpišite trajanje.' }).min(0.5, 'Trajanje mora biti od 0,5 do 200 ur.').max(200, 'Trajanje mora biti od 0,5 do 200 ur.'),
  earliest_start: z.string().min(1, 'Izberite datum.').refine((d) => d >= today(), 'Začetek ne sme biti v preteklosti.'),
  message: z.string().max(2000).optional(),
})
type Form = z.input<typeof quoteSchema>
type Data = z.output<typeof quoteSchema>

interface Props {
  jobId: string
  existing?: Quote | null
  onSaved: () => void
  onCancel?: () => void
}

export default function QuoteForm({ jobId, existing, onSaved, onCancel }: Props) {
  const [error, setError] = useState<string | null>(null)
  const { register, handleSubmit, formState, setError: setFieldError } = useForm<Form, unknown, Data>({
    resolver: zodResolver(quoteSchema),
    defaultValues: existing
      ? { price: existing.price, estimated_hours: existing.estimated_hours, earliest_start: existing.earliest_start, message: existing.message ?? '' }
      : undefined,
  })
  const err = formState.errors

  const onSubmit = async (data: Data) => {
    setError(null)
    try {
      if (existing) await api(`/quotes/${existing.id}`, { method: 'PUT', body: data })
      else await api(`/job-requests/${jobId}/quotes`, { method: 'POST', body: data })
      onSaved()
    } catch (e) {
      setError(applyServerError(e, setFieldError, ['price', 'estimated_hours', 'earliest_start', 'message']))
    }
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4" noValidate>
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label className="label" htmlFor="price">Cena z materialom (€) *</label>
          <input id="price" type="number" step="0.01" className="input" {...register('price')} />
          <p className="text-xs text-slate-500">Več kot 0 in največ 100.000 €</p>
          {err.price && <p className="error">{err.price.message}</p>}
        </div>
        <div>
          <label className="label" htmlFor="estimated_hours">Ocena trajanja (ure) *</label>
          <input id="estimated_hours" type="number" step="0.5" className="input" {...register('estimated_hours')} />
          <p className="text-xs text-slate-500">Od 0,5 do 200 ur</p>
          {err.estimated_hours && <p className="error">{err.estimated_hours.message}</p>}
        </div>
      </div>
      <div>
        <label className="label" htmlFor="earliest_start">Najzgodnejši začetek *</label>
        <input id="earliest_start" type="date" min={today()} className="input" {...register('earliest_start')} />
        <p className="text-xs text-slate-500">Ne sme biti v preteklosti</p>
        {err.earliest_start && <p className="error">{err.earliest_start.message}</p>}
      </div>
      <div>
        <label className="label" htmlFor="message">Sporočilo stranki</label>
        <textarea id="message" rows={4} className="input" {...register('message')} />
      </div>
      <p className="text-xs text-slate-500">
        Ponudbo lahko uredite ali umaknete, dokler je stranka ne sprejme.
        <br />
        Na eno povpraševanje lahko oddate samo eno ponudbo.
      </p>
      {error && <p className="error" role="alert">{error}</p>}
      <div className="flex justify-end gap-3 pt-4">
        {onCancel && (
          <button type="button" className="btn-light" onClick={onCancel}>Nazaj</button>
        )}
        <button className="btn" disabled={formState.isSubmitting}>{existing ? 'Shrani ponudbo' : 'Pošlji ponudbo'}</button>
      </div>
    </form>
  )
}
