# JWT in Argon2id za prijavo

Status: sprejeto

Datum: 2026-10-28

## Kontekst in problem

Uporabniki se morajo prijaviti, backend pa mora pri vsakem zahtevku vedeti, kdo je uporabnik in kakšno vlogo ima. Frontend je ločena SPA (glej [0001](0001-locen-frontend-in-rest-api.md)). Odločiti smo se morali, kako bo prijava delovala in kako bomo shranjevali gesla.

## Obravnavane možnosti

* JWT v glavi `Authorization` in gesla z Argon2id
* seje na strežniku s piškotkom in gesla z Argon2id
* JWT in gesla z bcrypt
* zunanji ponudnik prijave (npr. Keycloak ali Auth0)

## Odločitev

Izbrana možnost: "JWT in Argon2id", ker je žeton brez stanja in nam sej ni treba hraniti na strežniku ali v bazi. V žetonu je id uporabnika, vloga in čas veljavnosti, zato lahko Axum extractor pravice preveri brez dodatne poizvedbe za sejo. Pri SPA je to preprosto, ker frontend žeton samo pošlje v glavi. Seje s piškotki bi bile tudi v redu, ampak bi rabili tabelo za seje in zaščito pred CSRF. Zunanji ponudnik je za ta projekt preveč, ker bi dodal še en servis. Za gesla smo izbrali Argon2id, ker je trenutno priporočen algoritem (npr. pri OWASP) in je odporen tudi proti napadom z grafičnimi karticami, bcrypt pa je starejši in ima omejitev dolžine gesla na 72 bajtov.

### Posledice

* Dobro, ker backend ne hrani sej in je preverjanje hitro.
* Dobro, ker so gesla shranjena varno.
* Slabo, ker žetona ne moremo preklicati, preden poteče, zato ima kratko veljavnost (okoli 60 minut).
* Slabo, ker mora frontend paziti, kje hrani žeton.
