# 07 Pregled projekta

## 1.1 Stanje projekta
Popravi.si je spletna aplikacija, kjer stranka objavi povpraševanje za manjše delo v gospodinjstvu, mojstri pošljejo ponudbe, stranka izbere eno in nato spremlja potek dela.

| | |
|---|---|
| **Obseg** | MVP je narejen v celoti: 6 od 14 funkcionalnosti |
| **Zgradba** | baza PostgreSQL, backend v Rustu (Axum, SeaORM), frontend v Reactu |
| **Zagon** | Docker Compose |
| **Testi** | frontend: 18 testov, lint in build uspešni; backend: unit in integracijski testi |

## 1.2 Implementirane funkcionalnosti

| Funkcionalnost | Zgodba | Stanje |
|---|---|---|
| F1 Registracija in prijava | US-01 | ✓ narejeno |
| F2 Objava povpraševanja | US-02 | ✓ narejeno |
| F3 Seznam ustreznih del za izvajalca | US-03 | ✓ narejeno |
| F4 Oddaja, urejanje in umik ponudbe | US-04 | ✓ narejeno |
| F5 Primerjava in sprejem ponudbe | US-05 | ✓ narejeno |
| F6 Spremljanje statusa dela | US-06 | ✓ narejeno |


## 1.3 Manjkajoče funkcionalnosti

| Funkcionalnost | Zgodba |
|---|---|
| F7 Ocena izvajalca | US-07 |
| F8 Preverjanje izvajalcev | US-08 |
| F9 Javni profil izvajalca | US-09 |
| F10 Urejanje kategorij | US-10 |
| F11 Samodejni potek povpraševanja | US-11 |
| F12 Pregled za izvajalca | US-12 |
| F13 Moderiranje | US-13 |
| F14 Iskanje izvajalcev | US-14 |

## 1.4 Izpolnjeni sprejemni kriteriji
Vseh **30 od 30** kriterijev je izpolnjenih.

| Zgodba | Kriteriji | Kaj je pokrito |
|---|---|---|
| US-01 | 5 od 5 | zahtevana sporočila, Argon2id, žeton velja 60 minut |
| US-02 | 5 od 5 | preverjanje polj in proračuna, rok 14 dni, ulica skrita |
| US-03 | 5 od 5 | filter po statusu, kategoriji in poštni številki, 20 vrstic na stran, obvestilo za nepreverjene |
| US-04 | 5 od 5 | samo preverjen izvajalec, meje vrednosti, ena ponudba na povpraševanje |
| US-05 | 5 od 5 | stranka vidi samo svoje ponudbe, potrditev, sprejem v eni transakciji |
| US-06 | 5 od 5 | prehode dela samo pravi izvajalec, 409 pri neveljavnem prehodu, zgodovina, preklic z razlogom |