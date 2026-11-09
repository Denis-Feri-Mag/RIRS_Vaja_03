import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Link, useNavigate } from 'react-router-dom'
import { api } from '../lib/api'
import { applyServerError } from '../lib/forms'

const schema = z.object({
  email: z.string().email('Vpišite veljaven e-naslov.'),
  full_name: z.string().trim().min(1, 'Vpišite ime in priimek.').max(120),
  password: z.string().min(8, 'Geslo mora imeti vsaj 8 znakov.'),
  phone: z.string().max(30).optional(),
  role: z.enum(['CUSTOMER', 'PROVIDER'], { message: 'Izberite vlogo.' }),
})
type Form = z.infer<typeof schema>

export default function RegisterPage() {
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  const { register, handleSubmit, formState, setError: setFieldError } = useForm<Form>({ resolver: zodResolver(schema) })

  const onSubmit = async (data: Form) => {
    setError(null)
    try {
      await api('/auth/register', { method: 'POST', body: data })
      navigate('/prijava?registracija=1')
    } catch (e) {
      setError(applyServerError(e, setFieldError, ['email', 'full_name', 'password', 'phone', 'role']))
    }
  }
  const err = formState.errors

  return (
    <div className="card mx-auto mt-8 max-w-md p-6">
      <h1 className="mb-4 text-xl font-semibold text-sage-600">Registracija</h1>
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-4" noValidate>
        <div>
          <label className="label" htmlFor="email">E-naslov *</label>
          <input id="email" type="email" autoComplete="email" placeholder="ime@primer.si" className="input" {...register('email')} />
          {err.email && <p className="error">{err.email.message}</p>}
        </div>
        <div>
          <label className="label" htmlFor="full_name">Ime in priimek *</label>
          <input id="full_name" autoComplete="name" className="input" {...register('full_name')} />
          {err.full_name && <p className="error">{err.full_name.message}</p>}
        </div>
        <div>
          <label className="label" htmlFor="password">Geslo *</label>
          <input id="password" type="password" autoComplete="new-password" className="input" {...register('password')} />
          <p className="mt-1 text-xs text-slate-500">Vsaj 8 znakov</p>
          {err.password && <p className="error">{err.password.message}</p>}
        </div>
        <div>
          <label className="label" htmlFor="phone">Telefon</label>
          <input id="phone" type="tel" autoComplete="tel" placeholder="041 123 456" className="input" {...register('phone')} />
        </div>
        <fieldset>
          <legend className="label">Vloga *</legend>
          <div className="grid grid-cols-2 gap-3">
            {[
              ['CUSTOMER', 'Stranka', 'Iščem mojstra'],
              ['PROVIDER', 'Izvajalec', 'Ponujam storitve'],
            ].map(([value, name, hint]) => (
              <label
                key={value}
                className="flex items-start gap-2 rounded-md border border-slate-300 bg-slate-50 p-3 text-sm hover:border-sage-500 has-[:checked]:border-sage-500 has-[:checked]:bg-sage-50"
              >
                <input type="radio" value={value} aria-label={name} className="mt-0.5 accent-sage-600" {...register('role')} />
                <span>
                  <span className="block font-medium">{name}</span>
                  <span className="text-xs text-slate-500">{hint}</span>
                </span>
              </label>
            ))}
          </div>
          {err.role && <p className="error">{err.role.message}</p>}
        </fieldset>
        {error && <p className="error" role="alert">{error}</p>}
        <button className="btn w-full" disabled={formState.isSubmitting}>Ustvari račun</button>
      </form>
      <p className="mt-4 text-sm">
        Že imate račun? <Link to="/prijava" className="font-medium text-sage-700 underline">Prijava</Link>
      </p>
    </div>
  )
}
