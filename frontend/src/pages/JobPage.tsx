import { useQuery } from '@tanstack/react-query'
import { useParams } from 'react-router-dom'
import { api } from '../lib/api'
import { budget, jobStatusLabel, term } from '../lib/format'
import type { Job } from '../lib/types'

export default function JobPage() {
  const { id = '' } = useParams()
  const job = useQuery({ queryKey: ['job', id], queryFn: () => api<Job>(`/job-requests/${id}`) })

  if (job.isLoading) return <p>Nalagam...</p>
  if (job.error) return <p className="card">{(job.error as Error).message}</p>
  const j = job.data!

  return (
    <div className="space-y-4">
      <div className="card">
        <h1 className="text-xl font-semibold text-sage-600">{j.title}</h1>
        <p className="mb-2 text-sm text-slate-600">
          {j.category_name}, {j.postal_code} {j.city}, status: {jobStatusLabel[j.status]}
        </p>
        <p className="mb-3 whitespace-pre-line">{j.description}</p>
        <div className="grid gap-1 text-sm md:grid-cols-2">
          <p>Termin: {term(j.preferred_from, j.preferred_to)}</p>
          <p>Proračun: {budget(j.budget_min, j.budget_max)}</p>
          {j.address_line && <p>Naslov: {j.address_line}</p>}
        </div>
      </div>
    </div>
  )
}
