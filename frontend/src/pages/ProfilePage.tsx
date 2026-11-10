import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../lib/api'
import { applyServerError } from '../lib/forms'
import type { Category, ProviderProfile } from '../lib/types'

interface Form {
  business_name: string
  tax_number: string
  description: string
  service_city: string
  postal_codes: string
  category_ids: string[]
}

const verificationLabel = { PENDING: 'čaka na preverjanje', VERIFIED: 'preverjen', REJECTED: 'zavrnjen' }

export default function ProfilePage() {
  const qc = useQueryClient()
  const [msg, setMsg] = useState<string | null>(null)
  const profile = useQuery({ queryKey: ['profile'], queryFn: () => api<ProviderProfile>('/providers/me') })
  const categories = useQuery({ queryKey: ['categories'], queryFn: () => api<Category[]>('/categories') })
  const { register, handleSubmit, reset, formState, setError } = useForm<Form>()

  useEffect(() => {
    const p = profile.data
    if (p) {
      reset({
        business_name: p.business_name,
        tax_number: p.tax_number,
        description: p.description,
        service_city: p.service_city,
        postal_codes: p.service_postal_codes.join(', '),
        category_ids: p.category_ids,
      })
    }
  }, [profile.data, reset])

  const onSubmit = async (f: Form) => {
    setMsg(null)
    try {
      await api('/providers/me', {
        method: 'PUT',
        body: {
          business_name: f.business_name,
          tax_number: f.tax_number,
          description: f.description,
          service_city: f.service_city,
          service_postal_codes: f.postal_codes.split(/[\s,]+/).filter(Boolean),
          category_ids: f.category_ids || [],
        },
      })
      await qc.invalidateQueries()
      setMsg('Profil je shranjen.')
    } catch (e) {
      setMsg(applyServerError(e, setError, ['business_name', 'tax_number', 'service_city']))
    }
  }
  const err = formState.errors

  return (
    <div className="card max-w-2xl">
      <h1 className="mb-1 text-xl font-semibold text-sage-600">Profil izvajalca</h1>
      {profile.data && <p className="mb-4 text-sm text-slate-600">Status profila: {verificationLabel[profile.data.verification_status]}</p>}
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-3">
        <div>
          <label className="label" htmlFor="business_name">Naziv</label>
          <input id="business_name" className="input" {...register('business_name')} />
          {err.business_name && <p className="error">{err.business_name.message}</p>}
        </div>
        <div>
          <label className="label" htmlFor="tax_number">Davčna številka</label>
          <input id="tax_number" className="input" {...register('tax_number')} />
        </div>
        <div>
          <label className="label" htmlFor="description">Opis</label>
          <textarea id="description" className="input" rows={3} {...register('description')} />
        </div>
        <div>
          <label className="label" htmlFor="service_city">Kraj</label>
          <input id="service_city" className="input" {...register('service_city')} />
        </div>
        <div>
          <label className="label" htmlFor="postal_codes">Poštne številke, kjer delate, ločene z vejico</label>
          <input id="postal_codes" className="input" {...register('postal_codes')} />
        </div>
        <fieldset>
          <legend className="label">Kategorije</legend>
          {categories.data?.map((c) => (
            <label key={c.id} className="mr-4 inline-block text-sm">
              <input type="checkbox" value={c.id} {...register('category_ids')} /> {c.name}
            </label>
          ))}
        </fieldset>
        {msg && <p className="text-sm" role="status">{msg}</p>}
        <button className="btn" disabled={formState.isSubmitting}>Shrani</button>
      </form>
    </div>
  )
}
