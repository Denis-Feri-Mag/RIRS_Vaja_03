# Pravila za statuse na enem mestu v backendu

Status: sprejeto

Datum: 2026-10-28

## Kontekst in problem

Povpraševanje gre skozi več statusov, od OPEN do REVIEWED, lahko pa je tudi preklicano ali poteče. Vsak prehod ima pravila: kdo ga sme narediti (stranka, izvajalec, administrator) in iz katerega statusa. Vprašanje je bilo, kje naj se ta pravila preverjajo.

## Obravnavane možnosti

* ena funkcija v service sloju backenda, ki preveri prehod in pravice ter zapiše zgodovino
* preverjanje na frontendu
* preverjanje v vsakem handlerju posebej
* triggerji v bazi

## Odločitev

Izbrana možnost: "ena funkcija v service sloju backenda", ker frontendu ne moremo zaupati, saj lahko kdorkoli pošlje zahtevek mimo njega. Če bi preverjanje razpršili po handlerjih, bi se pravila hitro razlikovala med sabo in kakšen primer bi pozabili. Triggerji v bazi bi delovali, ampak je logiko v SQL težje testirati in brati, napake pa bi bile manj razumljive za uporabnika. Tako ena funkcija preveri, ali je prehod dovoljen in ali ga sme narediti ta uporabnik, nato pa v eni transakciji posodobi povpraševanje in doda zapis v `job_status_history`.

### Posledice

* Dobro, ker so vsa pravila na enem mestu in jih lahko pokrijemo z unit testi.
* Dobro, ker se vsaka sprememba zapiše v zgodovino in vemo, kdo je kaj naredil.
* Slabo, ker mora frontend nekatera pravila vseeno poznati, da ne prikaže gumbov, ki ne delujejo, in jih moramo vzdrževati na dveh mestih.
* Slabo, ker lahko ta funkcija sčasoma postane precej velika.
