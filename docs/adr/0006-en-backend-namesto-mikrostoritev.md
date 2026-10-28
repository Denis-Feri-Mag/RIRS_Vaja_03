# En backend namesto mikrostoritev

Status: sprejeto

Datum: 2026-10-28

## Kontekst in problem

Aplikacija ima več področij: uporabnike in prijavo, povpraševanja, ponudbe, ocene in administracijo. Odločiti smo se morali, ali bo backend en program ali pa ga razdelimo na več storitev.

## Obravnavane možnosti

* en backend (modularni monolit) z ločenimi crate-i v Cargo workspaceu
* mikrostoritve, npr. posebej za uporabnike, povpraševanja in ponudbe
* en backend brez delitve kode

## Odločitev

Izbrana možnost: "en backend z ločenimi crate-i", ker so za projekt te velikosti mikrostoritve samo dodatno delo. Rabili bi komunikacijo med storitvami, več baz ali skupno bazo, več Docker vsebnikov in bolj zapleteno postavitev. Sprejem ponudbe se dotakne povpraševanja in ponudb hkrati, kar je v enem backendu ena transakcija, pri mikrostoritvah pa bi bilo to precej težje. Da koda ne bi bila ena velika kopica, jo razdelimo na crate-e `api`, `service`, `entity` in `migration`.

### Posledice

* Dobro, ker je postavitev preprosta in je samo en program za zagon.
* Dobro, ker lahko uporabimo transakcije čez vse tabele.
* Slabo, ker posameznih delov ne moremo skalirati ločeno.
* Slabo, ker moramo sami paziti na meje med crate-i, da se ne začnejo prepletati.
