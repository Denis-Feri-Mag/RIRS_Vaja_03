# 05 Uporabniške zgodbe

Za vsako funkcionalnost iz `02-funkcionalnosti-in-obseg.md` smo napisali eno uporabniško zgodbo, skupaj jih je 14. Zgodbe za MVP (F1 do F6) so označene z "MVP". Pri vsaki so navedeni pogoji sprejemljivosti, po katerih bomo lahko preverili, ali je zgodba narejena. Statusi in imena polj so enaki kot v `03-podatkovni-model.md`.

Zgodbe smo poskušali držati majhne, da se jih da narediti v eni iteraciji. Kjer je zgodba odvisna od druge (npr. ponudbe ni brez povpraševanja), za testiranje uporabimo začetne podatke iz migracije, tako da jih lahko razvijamo neodvisno.

## US-01 Registracija in prijava (MVP)

Funkcionalnost: F1  
Vloga: obiskovalec

Kot obiskovalec želim ustvariti račun kot stranka ali izvajalec in se prijaviti, da lahko uporabljam funkcije, ki so namenjene moji vlogi.

Pogoji sprejemljivosti:
1. Obrazec za registracijo zahteva e-pošto, ime in priimek, geslo z vsaj 8 znaki ter izbiro vloge "Stranka" ali "Izvajalec". Vloge "Administrator" ni mogoče izbrati.
2. Če e-pošta že obstaja, se prikaže sporočilo "Račun s tem e-naslovom že obstaja." in račun se ne ustvari.
3. Geslo se v bazi shrani samo kot Argon2id hash v polju `password_hash`.
4. Po uspešni prijavi backend vrne JWT, ki velja 60 minut. Ko poteče, zahteve vrnejo 401 in uporabnik je preusmerjen na prijavo.
5. Pri napačnem geslu se prikaže "Napačen e-naslov ali geslo." Blokiran uporabnik (`is_active = false`) se ne more prijaviti.

## US-02 Objava povpraševanja (MVP)

Funkcionalnost: F2  
Vloga: stranka

Kot stranka želim opisati delo, ki ga potrebujem, da mi lahko ustrezni mojstri pošljejo ponudbe.

Pogoji sprejemljivosti:
1. Obvezna polja so kategorija, naslov (5 do 150 znakov), opis (vsaj 20 znakov), kraj in poštna številka (4 števke). Želeni termin in proračun sta neobvezna.
2. Če sta vpisana oba zneska proračuna, mora biti `budget_min` manjši ali enak `budget_max`, sicer se prikaže napaka pri polju.
3. Novo povpraševanje dobi status `OPEN`, `expires_at` je 14 dni po objavi, v `job_status_history` pa se zapiše prehod iz NULL v `OPEN`.
4. Izbrati je mogoče samo aktivne kategorije (`is_active = true`).
5. Ulica (`address_line`) se ne prikaže nikomur razen stranki, dokler ponudba ni sprejeta.

## US-03 Seznam ustreznih del za izvajalca (MVP)

Funkcionalnost: F3  
Vloga: izvajalec

Kot izvajalec želim videti samo povpraševanja iz svojih kategorij in s svojega območja, da ne izgubljam časa z deli, ki jih ne morem prevzeti.

Pogoji sprejemljivosti:
1. Seznam prikaže samo povpraševanja s statusom `OPEN` ali `QUOTED`, katerih kategorija je med izvajalčevimi kategorijami in poštna številka v `service_postal_codes`.
2. Povpraševanja so razvrščena od najnovejšega, po 20 na stran.
3. Vsaka vrstica prikazuje naslov, kategorijo, kraj, želeni termin, proračun (če obstaja) in število že oddanih ponudb.
4. Povpraševanja, na katera je izvajalec že poslal ponudbo, so označena z "Ponudba oddana".
5. Ko izvajalec še ni preverjen, se nad seznamom prikaže "Vaš profil še čaka na preverjanje. Ponudb še ne morete oddati."

## US-04 Oddaja ponudbe (MVP)

Funkcionalnost: F4  
Vloga: izvajalec

Kot izvajalec želim stranki poslati ponudbo s ceno in terminom, da me lahko izbere za delo.

Pogoji sprejemljivosti:
1. Ponudbo lahko odda samo izvajalec z `verification_status = VERIFIED`, drugače backend vrne 403.
2. Cena mora biti večja od 0 in največ 100.000 €, ocena trajanja od 0,5 do 200 ur, najzgodnejši začetek ne sme biti v preteklosti.
3. Na isto povpraševanje lahko izvajalec pošlje samo eno ponudbo. Pri drugem poskusu se prikaže "Na to povpraševanje ste že oddali ponudbo."
4. Prva ponudba spremeni status povpraševanja iz `OPEN` v `QUOTED` in to zapiše v zgodovino. Nove ponudbe statusa ne spremenijo več.
5. Dokler je ponudba `PENDING`, jo lahko izvajalec uredi ali umakne (`WITHDRAWN`).

## US-05 Primerjava in sprejem ponudbe (MVP)

Funkcionalnost: F5  
Vloga: stranka

Kot stranka želim videti vse ponudbe za svoje povpraševanje na enem mestu in izbrati eno, da se odločim za najboljšega mojstra.

Pogoji sprejemljivosti:
1. Stranka vidi ponudbe samo za svoja povpraševanja. Za tuja povpraševanja backend vrne 403.
2. Ponudbe so prikazane v tabeli s ceno, oceno trajanja, najzgodnejšim začetkom, povprečno oceno mojstra in številom opravljenih del. Razvrstiti jih je mogoče po ceni ali terminu.
3. Pred sprejemom se prikaže potrditev "Ali res želite sprejeti ponudbo? Ostale ponudbe bodo zavrnjene."
4. Ob sprejemu se v eni transakciji izbrana ponudba nastavi na `ACCEPTED`, ostale `PENDING` ponudbe na `REJECTED`, povpraševanje na `ACCEPTED`, `accepted_quote_id` pa na izbrano ponudbo.
5. Ko je ponudba sprejeta, gumb "Sprejmi" pri ostalih ponudbah ni več na voljo, izbrani mojster pa vidi naslov in telefon stranke.

## US-06 Spremljanje statusa dela (MVP)

Funkcionalnost: F6  
Vloga: izvajalec in stranka

Kot izvajalec želim označiti, da sem delo začel in ga končal, da stranka ves čas ve, kje je delo.

Pogoji sprejemljivosti:
1. Samo izvajalec sprejete ponudbe lahko spremeni status iz `ACCEPTED` v `IN_PROGRESS` in iz `IN_PROGRESS` v `COMPLETED`.
2. Prehod, ki ni v diagramu stanj (npr. iz `OPEN` v `COMPLETED`), backend zavrne s kodo 409 in sporočilom "Ta sprememba statusa ni dovoljena."
3. Vsaka sprememba doda vrstico v `job_status_history` s prejšnjim in novim statusom, uporabnikom in časom.
4. Stranka na strani dela vidi časovnico vseh sprememb, od najstarejše do najnovejše.
5. Stranka lahko prekliče delo samo pred statusom `IN_PROGRESS` in mora vpisati razlog (vsaj 10 znakov).

## US-07 Ocena izvajalca

Funkcionalnost: F7  
Vloga: stranka

Kot stranka želim po končanem delu oceniti mojstra, da drugi vedo, kakšne izkušnje sem imela.

Pogoji sprejemljivosti:
1. Oceno je mogoče oddati samo za delo s statusom `COMPLETED`, ki pripada stranki.
2. Ocena je celo število od 1 do 5, komentar je neobvezen in ima največ 1000 znakov.
3. Na eno delo je mogoča samo ena ocena. Po oddaji gre delo v status `REVIEWED`.
4. Po oddaji se izvajalcu ponovno izračunata `avg_rating` in `completed_jobs`.
5. Stranka lahko oceno popravi 7 dni po oddaji, potem je gumb "Uredi" skrit in backend vrne 403.

## US-08 Preverjanje izvajalca

Funkcionalnost: F8  
Vloga: administrator

Kot administrator želim pregledati podatke novih izvajalcev in jih potrditi ali zavrniti, da na platformi delajo samo pravi mojstri.

Pogoji sprejemljivosti:
1. Administrator vidi seznam profilov s statusom `PENDING`, najstarejši najprej, z nazivom, davčno številko in kategorijami.
2. Davčna številka mora imeti 8 števk, drugače je ni mogoče potrditi.
3. Ob potrditvi se nastavijo `verification_status = VERIFIED`, `verified_by` in `verified_at`.
4. Pri zavrnitvi je obvezen razlog. Izvajalec ga vidi na svojem profilu, status pa je `REJECTED`.
5. Zavrnjen izvajalec lahko popravi podatke, nakar profil spet dobi status `PENDING`.

## US-09 Javni profil izvajalca

Funkcionalnost: F9  
Vloga: obiskovalec in stranka

Kot stranka želim pogledati profil mojstra, da pred sprejemom ponudbe vidim, kako ga ocenjujejo drugi.

Pogoji sprejemljivosti:
1. Profil prikazuje naziv, opis, kategorije, kraj, povprečno oceno na eno decimalko in število opravljenih del.
2. Ocene so prikazane od najnovejše, po 10 na stran, skrite ocene (`is_hidden = true`) se ne prikažejo.
3. Telefon in e-pošta izvajalca na javnem profilu nista vidna.
4. Profil je dostopen tudi brez prijave, vendar samo za izvajalce s statusom `VERIFIED`.

## US-10 Urejanje kategorij

Funkcionalnost: F10  
Vloga: administrator

Kot administrator želim dodajati in urejati kategorije storitev, da stranke lahko izberejo pravo vrsto dela.

Pogoji sprejemljivosti:
1. Ime kategorije ima 2 do 80 znakov in mora biti unikatno. Pri podvojenem imenu se prikaže "Kategorija s tem imenom že obstaja."
2. `slug` se ustvari samodejno iz imena (npr. "Vodovodna dela" postane `vodovodna-dela`).
3. Kategorije ni mogoče izbrisati, lahko pa se deaktivira. Deaktivirana kategorija se ne pokaže pri novem povpraševanju, obstoječa povpraševanja pa ostanejo.
4. Do urejanja kategorij ima dostop samo administrator, ostali dobijo 403.

## US-11 Potek in preklic povpraševanja

Funkcionalnost: F11  
Vloga: stranka

Kot stranka želim, da stara povpraševanja brez izbrane ponudbe sama potečejo, da mojstri ne pošiljajo ponudb za dela, ki niso več aktualna.

Pogoji sprejemljivosti:
1. Opravilo v ozadju se zažene enkrat na uro in povpraševanjem s statusom `OPEN` ali `QUOTED`, katerih `expires_at` je v preteklosti, nastavi status `EXPIRED`.
2. Ob tem se vse ponudbe `PENDING` za ta povpraševanja nastavijo na `REJECTED`.
3. Vsak tak prehod se zapiše v zgodovino z opombo "Povpraševanje je poteklo".
4. Poteklo povpraševanje se ne prikaže več na seznamu ustreznih del (US-03).
5. Stranka lahko povpraševanje s statusom `OPEN` ali `QUOTED` tudi sama prekliče (`CANCELLED`), pri čemer mora vpisati razlog.

## US-12 Pregled za izvajalca

Funkcionalnost: F12  
Vloga: izvajalec

Kot izvajalec želim imeti pregled nad svojimi ponudbami in deli, da vem, kaj me čaka ta teden.

Pogoji sprejemljivosti:
1. Pregled prikazuje število ponudb po statusih `PENDING`, `ACCEPTED`, `REJECTED` in `WITHDRAWN`.
2. Prikazan je delež sprejetih ponudb v zadnjih 90 dneh v odstotkih, zaokrožen na celo število.
3. Seznam prihajajočih del vsebuje dela s statusom `ACCEPTED` ali `IN_PROGRESS`, razvrščena po `earliest_start`.
4. Prikazan je seštevek cen sprejetih ponudb v tekočem mesecu.

## US-13 Moderiranje

Funkcionalnost: F13  
Vloga: administrator

Kot administrator želim skriti neprimerne ocene in blokirati uporabnike, ki zlorabljajo platformo, da ostane uporabna za vse.

Pogoji sprejemljivosti:
1. Administrator lahko oceno skrije (`is_hidden = true`) ali jo spet pokaže. Skrita ocena se ne upošteva v `avg_rating`.
2. Blokiran uporabnik (`is_active = false`) se ne more prijaviti, njegovi obstoječi žetoni pa vrnejo 401.
3. Blokiranje ne izbriše podatkov, povpraševanja in ponudbe uporabnika ostanejo v bazi.
4. Administrator ne more blokirati samega sebe.

## US-14 Iskanje izvajalcev

Funkcionalnost: F14  
Vloga: obiskovalec in stranka

Kot stranka želim poiskati mojstre po kategoriji in kraju, da lahko pred objavo povpraševanja vidim, kdo dela v moji okolici.

Pogoji sprejemljivosti:
1. Filtri so kategorija, kraj in minimalna ocena (1 do 5). Vsi so neobvezni.
2. Rezultati vsebujejo samo preverjene in aktivne izvajalce, po 20 na stran.
3. Privzeto so razvrščeni po povprečni oceni, od najvišje.
4. Če ni zadetkov, se prikaže "Ni izvajalcev, ki bi ustrezali iskanju."

## Povzetek

| Zgodba | Funkcionalnost | Vloga | MVP |
|---|---|---|---|
| US-01 | F1 | obiskovalec | da |
| US-02 | F2 | stranka | da |
| US-03 | F3 | izvajalec | da |
| US-04 | F4 | izvajalec | da |
| US-05 | F5 | stranka | da |
| US-06 | F6 | izvajalec, stranka | da |
| US-07 | F7 | stranka | |
| US-08 | F8 | administrator | |
| US-09 | F9 | obiskovalec, stranka | |
| US-10 | F10 | administrator | |
| US-11 | F11 | stranka | |
| US-12 | F12 | izvajalec | |
| US-13 | F13 | administrator | |
| US-14 | F14 | obiskovalec, stranka | |
