# Popravi.si

Projekt pri predmetu RIRS na FERI Maribor.

Popravi.si je spletna aplikacija za manjša dela v gospodinjstvu, npr. popravilo pipe ali pleskanje sobe. Stranka objavi povpraševanje, lokalni mojstri ji pošljejo ponudbe, ona izbere eno, potem pa oba spremljata, kako delo napreduje. Na koncu stranka mojstra še oceni.

## Tehnologije

- frontend: React, TypeScript, Vite
- backend: Rust (Axum, Tokio, SeaORM)
- baza: PostgreSQL 18
- prijava: JWT, gesla z Argon2id
- lokalni zagon: Docker Compose

## Lokalni zagon

Potrebna sta Docker in Docker Compose.

```bash
cp .env.example .env
docker compose up --build
```

V `.env` zamenjaj `JWT_SECRET` in geslo administratorja. Frontend je na http://localhost:5173, API na http://localhost:8080/api/v1, Swagger UI pa na http://localhost:8080/swagger-ui.

Če je v `.env` nastavljen `DEMO_PASSWORD`, se ob zagonu ustvarijo testni računi s tem geslom: stranka `stranka@popravi.test`, preverjen mojster `mojster@popravi.test` in nepreverjen mojster `novi.mojster@popravi.test`. Mojstra pokrivata vodovodna dela in elektriko za poštni številki 2000 in 2341.

Brez Dockerja za backend in frontend rabiš Rust in Node.js 20. Baza naj teče v Dockerju:

```bash
docker compose up db
cd backend && cargo run -p api
cd frontend && npm install && npm run dev
```

## Testi

```bash
cd backend && cargo test
cd frontend && npm test
```

Backend testi za API rabijo prazno bazo PostgreSQL, npr. `TEST_DATABASE_URL=postgres://popravi:popravi@localhost:5432/postgres cargo test`. Vsak test si ustvari svojo bazo. Brez te spremenljivke se ti testi preskočijo.

## Dokumentacija

Vsa dokumentacija je v mapi [docs](docs/):

- [01 Zasnova rešitve](docs/01-zasnova-resitve.md)
- [02 Funkcionalnosti in obseg](docs/02-funkcionalnosti-in-obseg.md)
- [03 Podatkovni model](docs/03-podatkovni-model.md)
- [04 Arhitektura in tehnološki sklad](docs/04-arhitektura-in-tehnoloski-sklad.md)
- [05 Uporabniške zgodbe](docs/05-uporabniske-zgodbe.md)
- [06 Wireframes](docs/06-wireframes.md)
- [Arhitekturne odločitve (ADR)](docs/adr/)
