import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import RegisterPage from '../pages/RegisterPage'
import { renderPage } from './utils'

afterEach(() => localStorage.clear())

describe('US-01 registracija', () => {
  it('ponudi samo vlogi stranka in izvajalec', () => {
    renderPage(<RegisterPage />)
    expect(screen.getByLabelText('Stranka')).toBeInTheDocument()
    expect(screen.getByLabelText('Izvajalec')).toBeInTheDocument()
    expect(screen.queryByLabelText(/Administrator/)).toBeNull()
  })

  it('pokaže napako za podvojen e-naslov', async () => {
    globalThis.fetch = vi.fn(async () =>
      new Response(JSON.stringify({ code: 'EMAIL_TAKEN', message: 'Račun s tem e-naslovom že obstaja.' }), { status: 409 }),
    ) as typeof fetch
    renderPage(<RegisterPage />)
    const u = userEvent.setup()
    await u.type(screen.getByLabelText(/^E-naslov/), 'ana@primer.si')
    await u.type(screen.getByLabelText(/^Ime in priimek/), 'Ana Novak')
    await u.type(screen.getByLabelText(/^Geslo/), 'geslo1234')
    await u.click(screen.getByLabelText('Stranka'))
    await u.click(screen.getByRole('button', { name: 'Ustvari račun' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('Račun s tem e-naslovom že obstaja.')
  })
})
