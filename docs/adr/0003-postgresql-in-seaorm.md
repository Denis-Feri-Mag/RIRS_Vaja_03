# PostgreSQL in SeaORM

Status: sprejeto

Datum: 2026-10-28

## Kontekst in problem

Podatki v aplikaciji so med sabo močno povezani: uporabniki, profili izvajalcev, kategorije, povpraševanja, ponudbe, ocene in zgodovina statusov. Ko stranka sprejme ponudbo, se mora hkrati spremeniti več vrstic (sprejeta ponudba, ostale ponudbe, status povpraševanja, zapis v zgodovino). Potrebujemo bazo in način, kako bo backend dostopal do nje.

## Obravnavane možnosti

* PostgreSQL 18 s SeaORM
* PostgreSQL 18 z Dieselom
* PostgreSQL 18 s sqlx (ročno pisan SQL)
* MySQL
* MongoDB

## Odločitev

Izbrana možnost: "PostgreSQL 18 s SeaORM", ker so naši podatki relacijski in potrebujemo transakcije. Del pravil lahko preverja kar baza, npr. delni unikatni indeks, ki dovoli samo eno sprejeto ponudbo na povpraševanje, in omejitev ene ocene na delo. MongoDB smo izpustili, ker bi morali povezave med dokumenti in te omejitve skrbeti sami v kodi. MySQL bi deloval, ampak PostgreSQL podpira delne indekse, ki jih mi uporabljamo. SeaORM smo izbrali, ker je asinhron in deluje s Tokio, ima pa tudi svoje migracije (sea-orm-migration), ki so v repozitoriju, zato ima vsak enako shemo. Diesel je sinhron, kar se z Axumom slabše ujema. Samo sqlx bi pomenil več ročnega pisanja SQL, SeaORM pa sqlx že uporablja pod sabo.

### Posledice

* Dobro, ker baza sama preprečuje napačne podatke (tuji ključi, unikatni indeksi).
* Dobro, ker so migracije v repozitoriju in se izvedejo ob zagonu.
* Slabo, ker je SeaORM še ena knjižnica, ki se je moramo naučiti, in pri bolj zapletenih poizvedbah včasih vseeno rabimo ročni SQL.
* Slabo, ker potrebujemo zagnan PostgreSQL tudi za lokalni razvoj in teste.
