# Docker Compose za lokalni zagon

Status: sprejeto

Datum: 2026-10-28

## Kontekst in problem

Za zagon aplikacije potrebujemo PostgreSQL, backend in frontend. Vsak, ki dela na projektu, ima lahko drug sistem in druge različice programov. Želeli smo, da se projekt zažene na enak način pri vseh.

## Obravnavane možnosti

* Docker Compose s storitvami `db`, `backend` in `frontend`
* ročna namestitev PostgreSQL, Rusta in Node.js po navodilih v README
* razvoj v oblaku, npr. GitHub Codespaces

## Odločitev

Izbrana možnost: "Docker Compose", ker lahko vsak z enim ukazom (`docker compose up --build`) zažene enako okolje z isto različico PostgreSQL 18 in mu baze ni treba ročno nameščati. Pri ročni namestitvi bi imel vsak malo drugačne različice in nastavitve, kar pogosto pomeni napake, ki se pojavijo samo pri enem. Codespaces bi rešil isto stvar, ampak je odvisen od povezave in omejenih brezplačnih ur. Nastavitve, kot sta geslo za bazo in `JWT_SECRET`, so v datoteki `.env`, ki ni v repozitoriju.

### Posledice

* Dobro, ker je zagon enak pri vseh in hiter za novega razvijalca.
* Dobro, ker lahko med razvojem v Dockerju poganjamo samo bazo, backend in frontend pa lokalno.
* Slabo, ker mora imeti vsak nameščen Docker, ki porabi kar nekaj pomnilnika.
* Slabo, ker je prevajanje Rusta v Dockerju počasnejše, če ne nastavimo predpomnjenja odvisnosti.
