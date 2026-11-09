import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'
import { api, ApiError } from '../lib/api'
import { useAuth } from '../lib/auth'
import type { User } from '../lib/types'

const schema = z.object({
  email: z.string().min(1, 'Vpišite e-naslov.'),
  password: z.string().min(1, 'Vpišite geslo.'),
})
type Form = z.infer<typeof schema>

export default function LoginPage() {
  const { login } = useAuth()
  const navigate = useNavigate()
  const [params] = useSearchParams()
  const [error, setError] = useState<string | null>(null)
  const { register, handleSubmit, formState } = useForm<Form>({ resolver: zodResolver(schema) })

  const onSubmit = async (data: Form) => {
    setError(null)
    try {
      const res = await api<{ token: string; user: User }>('/auth/login', { method: 'POST', body: data })
      login(res.token, res.user)
      navigate('/')
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Prijava ni uspela.')
    }
  }

  return (
    <div className="card mx-auto mt-8 max-w-md p-6">
      <h1 className="mb-4 text-xl font-semibold text-sage-600">Prijava</h1>
      {params.get('registracija') && <p className="mb-3 rounded border border-sage-200 bg-sage-50 text-sage-800 p-2 text-sm">Račun je ustvarjen, zdaj se lahko prijavite.</p>}
      {params.get('potekla') && <p className="mb-3 rounded border border-amber-300 bg-amber-50 text-amber-900 p-2 text-sm">Prijava je potekla, prijavite se znova.</p>}
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-4" noValidate>
        <div>
          <label className="label" htmlFor="email">E-naslov</label>
          <input id="email" type="email" autoComplete="email" placeholder="ime@primer.si" className="input" {...register('email')} />
          {formState.errors.email && <p className="error">{formState.errors.email.message}</p>}
        </div>
        <div>
          <label className="label" htmlFor="password">Geslo</label>
          <input id="password" type="password" autoComplete="current-password" className="input" {...register('password')} />
          {formState.errors.password && <p className="error">{formState.errors.password.message}</p>}
        </div>
        {error && <p className="error" role="alert">{error}</p>}
        <button className="btn w-full" disabled={formState.isSubmitting}>Prijava</button>
      </form>
      <p className="mt-4 text-sm">
        Še nimate računa? <Link to="/registracija" className="font-medium text-sage-700 underline">Registracija</Link>
      </p>
    </div>
  )
}
