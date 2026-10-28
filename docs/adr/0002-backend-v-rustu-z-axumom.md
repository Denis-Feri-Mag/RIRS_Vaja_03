# Backend v Rustu z Axumom

Status: sprejeto

Datum: 2026-10-28

## Kontekst in problem

Backend vsebuje vso poslovno logiko, najbolj pomembna pa so pravila za statuse povpraševanja (OPEN, QUOTED, ACCEPTED, IN_PROGRESS, COMPLETED, REVIEWED, CANCELLED, EXPIRED). Napaka pri prehodih bi pomenila, da npr. mojster začne delo, ki mu ni bilo dodeljeno. Izbrati smo morali jezik in ogrodje za backend.

## Obravnavane možnosti

* Rust z Axumom (na Tokio)
* Rust z Actix Web
* Node.js z Expressom (TypeScript)
* Go s standardno knjižnico ali Ginom

## Odločitev

Izbrana možnost: "Rust z Axumom", ker veliko napak ujame že prevajalnik. Statuse zapišemo kot enum in prevajalnik opozori, če v `match` kakšen status pozabimo obravnavati, kar je pri našem poteku dela zelo uporabno. Node.js bi bil hitrejši za začetek, ampak v JavaScriptu oziroma TypeScriptu tega preverjanja ni v taki meri. Go je preprostejši, vendar nima enumov s podatki in tako strogega preverjanja. Med ogrodji v Rustu smo izbrali Axum, ker je neposredno zgrajen na Tokio in Tower, tako da lahko za CORS, logiranje in preverjanje JWT uporabimo obstoječi middleware iz tower-http. Actix Web je tudi dober, ampak ima svoj sistem middleware, Axum pa se nam je zdel bolj preprost za uporabo.

### Posledice

* Dobro, ker prevajalnik ujame veliko napak, preden program sploh zaženemo.
* Dobro, ker je backend hiter in porabi malo pomnilnika.
* Slabo, ker se Rusta učimo dlje kot npr. Node.js in je razvoj na začetku počasnejši.
* Slabo, ker se projekt v Rustu prevaja dlje, kar se pozna med razvojem.
