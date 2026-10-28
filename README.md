# Popravi.si

Projekt pri predmetu RIRS na FERI Maribor.

Popravi.si je spletna aplikacija za manjša dela v gospodinjstvu, npr. popravilo pipe ali pleskanje sobe. Stranka objavi povpraševanje, lokalni mojstri ji pošljejo ponudbe, ona izbere eno, potem pa oba spremljata, kako delo napreduje. Na koncu stranka mojstra še oceni.

## Tehnologije

- frontend: React, TypeScript, Vite
- backend: Rust (Axum, Tokio, SeaORM)
- baza: PostgreSQL 18
- prijava: JWT, gesla z Argon2id
- lokalni zagon: Docker Compose

Mapi `frontend/` in `backend/` sta za zdaj še prazni.

## Dokumentacija

Vsa dokumentacija je v mapi [docs](docs/):

- [01 Zasnova rešitve](docs/01-zasnova-resitve.md)
- [02 Funkcionalnosti in obseg](docs/02-funkcionalnosti-in-obseg.md)
- [03 Podatkovni model](docs/03-podatkovni-model.md)
- [04 Arhitektura in tehnološki sklad](docs/04-arhitektura-in-tehnoloski-sklad.md)
- [05 Uporabniške zgodbe](docs/05-uporabniske-zgodbe.md)
- [06 Wireframes](docs/06-wireframes.md)
- [Arhitekturne odločitve (ADR)](docs/adr/)
