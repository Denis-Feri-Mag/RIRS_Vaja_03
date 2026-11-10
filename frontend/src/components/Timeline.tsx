import { dateTime, jobStatusLabel, jobStatusText } from '../lib/format'
import type { HistoryItem, JobStatus } from '../lib/types'

const steps: JobStatus[] = ['OPEN', 'QUOTED', 'ACCEPTED', 'IN_PROGRESS', 'COMPLETED', 'REVIEWED']

export function StatusBar({ status, activeQuotes }: { status: JobStatus; activeQuotes?: number }) {
  const current = steps.indexOf(status)
  return (
    <ol className="mb-4 flex flex-wrap gap-1 text-xs">
      {steps.map((s, i) => (
        <li key={s} className={`rounded px-2 py-1 ${i === current ? 'bg-sage-600 text-white' : i < current ? 'bg-sage-100 text-sage-800' : 'bg-slate-100 text-slate-500'}`}>
          {s === status ? jobStatusText(s, activeQuotes) : jobStatusLabel[s]}
        </li>
      ))}
      {current === -1 && <li className="rounded bg-red-100 px-2 py-1">{jobStatusLabel[status]}</li>}
    </ol>
  )
}

export function Stepper({ status }: { status: JobStatus }) {
  const current = steps.indexOf(status)
  return (
    <ol className="flex">
      {steps.map((s, i) => {
        const done = current >= 0 && i <= current
        return (
          <li key={s} className="relative flex flex-1 flex-col items-center text-center">
            {i > 0 && (
              <span className={`absolute right-1/2 top-3 h-0.5 w-full ${done ? 'bg-sage-500' : 'bg-slate-300'}`} />
            )}
            <span
              className={`relative z-[1] h-6 w-6 rounded-full border-2 ${done ? 'border-sage-500 bg-sage-500' : 'border-slate-400 bg-white'}`}
            />
            <span className={`mt-2 text-sm ${i === current ? 'font-semibold text-sage-700' : done ? '' : 'text-slate-500'}`}>
              {jobStatusLabel[s]}
            </span>
          </li>
        )
      })}
    </ol>
  )
}

export default function Timeline({ items }: { items: HistoryItem[] }) {
  return (
    <ol className="space-y-4 text-sm">
      {items.map((h, i) => (
        <li key={i} className="grid grid-cols-[9rem_1fr] gap-3">
          <span className="text-slate-500">{dateTime(h.changed_at)}</span>
          <div>
            <p className="font-semibold">
              {h.from_status ? `${jobStatusLabel[h.from_status]} → ` : ''}
              {jobStatusLabel[h.to_status]}
            </p>
            <p className="text-slate-500">{h.changed_by_name}</p>
            {h.note && <p className="text-slate-600">{h.note}</p>}
          </div>
        </li>
      ))}
    </ol>
  )
}
