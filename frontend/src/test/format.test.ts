import { date, jobStatusText, today } from '../lib/format'

describe('datumi', () => {
  it('datum brez ure se ne premakne zaradi časovnega pasu brskalnika', () => {
    const old = process.env.TZ
    process.env.TZ = 'America/Los_Angeles'
    expect(date('2026-11-10')).toBe('10. 11. 2026')
    process.env.TZ = old
  })
  it('danes je po času v Ljubljani', () => {
    expect(today(new Date('2026-11-09T23:30:00Z'))).toBe('2026-11-10')
    expect(today(new Date('2026-11-09T22:30:00Z'))).toBe('2026-11-09')
  })
})

describe('status povpraševanja', () => {
  it('QUOTED brez aktivnih ponudb', () => {
    expect(jobStatusText('QUOTED', 0)).toBe('Čaka na ponudbe')
    expect(jobStatusText('QUOTED', 1)).toBe('Prejete ponudbe')
    expect(jobStatusText('OPEN', 0)).toBe('Objavljeno')
  })
})
