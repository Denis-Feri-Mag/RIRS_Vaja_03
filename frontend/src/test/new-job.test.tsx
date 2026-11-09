import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import NewJobPage from '../pages/NewJobPage'
import { mockFetch, renderPage } from './utils'

beforeEach(() => mockFetch({ '/categories': [{ id: 'plumbing', name: 'Vodovodna dela' }] }))

it('ohrani razporeditev, oznake in gumbe iz WF-01', () => {
  const { container } = renderPage(<NewJobPage />)
  const left = container.querySelector('.request-required')!
  const right = container.querySelector('.request-details')!
  expect(Array.from(left.querySelectorAll('label')).map((label) => label.textContent)).toEqual([
    'Kategorija *', 'Naslov *', 'Opis *', 'Kraj *', 'Poštna številka *',
  ])
  expect(Array.from(right.querySelectorAll('label')).map((label) => label.textContent)).toEqual([
    'Želeni termin od', 'do', 'Proračun od (€)', 'Proračun do (€)', 'Ulica in hišna številka',
  ])
  expect(screen.queryByText('Neobvezno')).not.toBeInTheDocument()
  expect(screen.getByText('Opišite delo. Ponudbe vam bodo poslali preverjeni mojstri iz vaše okolice.')).toBeInTheDocument()
  expect(screen.getByText('5 do 150 znakov')).toBeInTheDocument()
  expect(screen.getByText('Vsaj 20 znakov')).toBeInTheDocument()
  expect(screen.getByLabelText('Kraj *').parentElement?.parentElement).toBe(screen.getByLabelText('Poštna številka *').parentElement?.parentElement)
  expect(screen.getByLabelText('Želeni termin od').parentElement?.parentElement).toBe(screen.getByLabelText('do').parentElement?.parentElement)
  expect(right).toContainElement(screen.getByText('Na voljo v kasnejši različici'))
  expect(container.querySelector('input[type="file"]')).toBeNull()
  expect(right).toContainElement(screen.getByText('Povpraševanje bo odprto 14 dni (status OPEN).'))
  expect(within(right as HTMLElement).getAllByRole('button').map((button) => button.textContent)).toEqual(['Prekliči', 'Objavi povpraševanje'])
  expect(screen.getByRole('button', { name: 'Prekliči' })).toHaveAttribute('type', 'button')
})

it('ohrani validacijo in oddajo brez neobveznih podatkov', async () => {
  const calls = mockFetch({ '/categories': [{ id: 'plumbing', name: 'Vodovodna dela' }], '/job-requests': { id: 'job1' } })
  renderPage(<NewJobPage />)
  const user = userEvent.setup()
  await user.click(screen.getByRole('button', { name: 'Objavi povpraševanje' }))
  expect(await screen.findByText('Izberite kategorijo.')).toBeInTheDocument()
  expect(calls.some((call) => call.init?.method === 'POST')).toBe(false)
  await screen.findByRole('option', { name: 'Vodovodna dela' })
  await user.selectOptions(screen.getByLabelText('Kategorija *'), 'plumbing')
  await user.type(screen.getByLabelText('Naslov *'), 'Pušča kuhinjska pipa')
  await user.type(screen.getByLabelText('Opis *'), 'Pipa v kuhinji kaplja že teden dni.')
  await user.type(screen.getByLabelText('Kraj *'), 'Maribor')
  await user.type(screen.getByLabelText('Poštna številka *'), '2000')
  await user.click(screen.getByRole('button', { name: 'Objavi povpraševanje' }))
  await waitFor(() => expect(calls.some((call) => call.init?.method === 'POST')).toBe(true))
  const posted = calls.find((call) => call.init?.method === 'POST')!
  expect(JSON.parse(posted.init!.body as string)).toEqual({
    category_id: 'plumbing', title: 'Pušča kuhinjska pipa', description: 'Pipa v kuhinji kaplja že teden dni.', city: 'Maribor', postal_code: '2000',
  })
})
