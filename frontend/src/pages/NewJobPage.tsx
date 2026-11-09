import { useState } from 'react'
import './NewJobPage.css'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { api } from '../lib/api'
import { applyServerError, optionalNumber, optionalString } from '../lib/forms'
import type { Category, Job } from '../lib/types'

const money = z.preprocess(optionalNumber, z.number().min(0, 'Znesek ne sme biti negativen.').max(100000, 'Največ 100.000 €.').optional())

export const jobSchema = z
  .object({
    category_id: z.string().min(1, 'Izberite kategorijo.'),
    title: z.string().trim().min(5, 'Naslov mora imeti od 5 do 150 znakov.').max(150, 'Naslov mora imeti od 5 do 150 znakov.'),
    description: z.string().trim().min(20, 'Opis mora imeti vsaj 20 znakov.'),
    city: z.string().trim().min(1, 'Vpišite kraj.').max(100),
    postal_code: z.string().regex(/^\d{4}$/, 'Poštna številka mora imeti 4 števke.'),
    address_line: z.preprocess(optionalString, z.string().max(200).optional()),
    preferred_from: z.preprocess(optionalString, z.string().optional()),
    preferred_to: z.preprocess(optionalString, z.string().optional()),
    budget_min: money,
    budget_max: money,
  })
  .refine((d) => d.budget_min == null || d.budget_max == null || d.budget_min <= d.budget_max, {
    path: ['budget_max'],
    message: 'Najnižji znesek mora biti manjši ali enak najvišjemu.',
  })
  .refine((d) => !d.preferred_from || !d.preferred_to || d.preferred_from <= d.preferred_to, {
    path: ['preferred_to'],
    message: 'Konec termina ne sme biti pred začetkom.',
  })
type Form = z.input<typeof jobSchema>
type Data = z.output<typeof jobSchema>

const fields = ['category_id', 'title', 'description', 'city', 'postal_code', 'address_line', 'preferred_from', 'preferred_to', 'budget_min', 'budget_max']

export default function NewJobPage() {
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  const categories = useQuery({ queryKey: ['categories'], queryFn: () => api<Category[]>('/categories') })
  const { register, handleSubmit, formState, setError: setFieldError } = useForm<Form, unknown, Data>({ resolver: zodResolver(jobSchema) })
  const err = formState.errors

  const onSubmit = async (data: Data) => {
    setError(null)
    try {
      const job = await api<Job>('/job-requests', { method: 'POST', body: data })
      navigate(`/povprasevanja/${job.id}`)
    } catch (e) {
      setError(applyServerError(e, setFieldError, fields))
    }
  }

  return (
    <div className="request-page">
      <h1 className="text-[22px] font-semibold text-sage-600">Novo povpraševanje</h1>
      <p className="request-intro">Opišite delo. Ponudbe vam bodo poslali preverjeni mojstri iz vaše okolice.</p>
      <form onSubmit={handleSubmit(onSubmit)} noValidate>
        <div className="request-columns">
          <div className="request-required">
            <div>
              <label className="label" htmlFor="category_id">Kategorija *</label>
              <select id="category_id" className="input" {...register('category_id')}>
                <option value="">Izberite...</option>
                {categories.data?.map((c) => (
                  <option key={c.id} value={c.id}>{c.name}</option>
                ))}
              </select>
              {err.category_id && <p className="error">{err.category_id.message}</p>}
            </div>
            <div>
              <label className="label" htmlFor="title">Naslov *</label>
              <input id="title" className="input" {...register('title')} />
              <p className="mt-1 text-xs text-slate-500">5 do 150 znakov</p>
              {err.title && <p className="error">{err.title.message}</p>}
            </div>
            <div>
              <label className="label" htmlFor="description">Opis *</label>
              <textarea id="description" rows={4} className="input" {...register('description')} />
              <p className="mt-1 text-xs text-slate-500">Vsaj 20 znakov</p>
              {err.description && <p className="error">{err.description.message}</p>}
            </div>
            <div className="request-pair">
              <div>
                <label className="label" htmlFor="city">Kraj *</label>
                <input id="city" className="input" {...register('city')} />
                {err.city && <p className="error">{err.city.message}</p>}
              </div>
              <div>
                <label className="label" htmlFor="postal_code">Poštna številka *</label>
                <input id="postal_code" className="input" inputMode="numeric" {...register('postal_code')} />
                {err.postal_code && <p className="error">{err.postal_code.message}</p>}
              </div>
            </div>
          </div>
          <div className="request-details">
            <div className="request-pair">
              <div>
                <label className="label" htmlFor="preferred_from">Želeni termin od</label>
                <input id="preferred_from" type="date" className="input" {...register('preferred_from')} />
              </div>
              <div>
                <label className="label" htmlFor="preferred_to">do</label>
                <input id="preferred_to" type="date" className="input" {...register('preferred_to')} />
                {err.preferred_to && <p className="error">{err.preferred_to.message}</p>}
              </div>
            </div>
            <div className="request-pair">
              <div>
                <label className="label" htmlFor="budget_min">Proračun od (€)</label>
                <input id="budget_min" type="number" step="0.01" className="input" {...register('budget_min')} />
                {err.budget_min && <p className="error">{err.budget_min.message}</p>}
              </div>
              <div>
                <label className="label" htmlFor="budget_max">Proračun do (€)</label>
                <input id="budget_max" type="number" step="0.01" className="input" {...register('budget_max')} />
                {err.budget_max && <p className="error">{err.budget_max.message}</p>}
              </div>
            </div>
            <div>
              <label className="label" htmlFor="address_line">Ulica in hišna številka</label>
              <input id="address_line" className="input" {...register('address_line')} />
              <p className="mt-1 text-xs text-slate-500">Vidi jo samo mojster, čigar ponudbo sprejmete.</p>
            </div>
            <div className="request-photos">
              <p className="label">Fotografije</p>
              <div className="request-photo-placeholder">Na voljo v kasnejši različici</div>
            </div>
            <div className="request-notice">
              <p>Povpraševanje bo odprto 14 dni (status OPEN).</p>
              <p>Če v tem času ne sprejmete nobene ponudbe, poteče.</p>
            </div>
            {error && <p className="error" role="alert">{error}</p>}
            <div className="request-actions">
              <button type="button" className="btn-light" onClick={() => navigate(-1)}>Prekliči</button>
              <button className="btn" disabled={formState.isSubmitting}>Objavi povpraševanje</button>
            </div>
          </div>
        </div>
      </form>
    </div>
  )
}
