import type { JobStatus } from './types'

export const jobStatusLabel: Record<JobStatus, string> = {
  OPEN: 'Objavljeno',
  QUOTED: 'Prejete ponudbe',
  ACCEPTED: 'Sprejeto',
  IN_PROGRESS: 'V teku',
  COMPLETED: 'Zaključeno',
  REVIEWED: 'Ocenjeno',
  CANCELLED: 'Preklicano',
  EXPIRED: 'Poteklo',
}

export const euro = (v: number) => new Intl.NumberFormat('sl-SI', { style: 'currency', currency: 'EUR' }).format(v)

export const TIME_ZONE = 'Europe/Ljubljana'

/** Date only values like 2026-11-10 are shown as they are, without time zone conversion. */
export function date(v: string | null) {
  if (!v) return ''
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(v)
  if (m) return `${Number(m[3])}. ${Number(m[2])}. ${m[1]}`
  return new Date(v).toLocaleDateString('sl-SI', { timeZone: TIME_ZONE })
}

export const dateTime = (v: string) => new Date(v).toLocaleString('sl-SI', { timeZone: TIME_ZONE })

export function budget(min: number | null, max: number | null) {
  if (min != null && max != null) return `${euro(min)} do ${euro(max)}`
  if (min != null) return `od ${euro(min)}`
  if (max != null) return `do ${euro(max)}`
  return 'ni naveden'
}

export function term(from: string | null, to: string | null) {
  if (from && to) return `${date(from)} do ${date(to)}`
  if (from) return `od ${date(from)}`
  if (to) return `do ${date(to)}`
  return 'po dogovoru'
}
