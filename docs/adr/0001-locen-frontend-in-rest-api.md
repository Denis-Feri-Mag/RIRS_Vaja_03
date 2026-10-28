# Ločen frontend in REST API

Status: sprejeto

Datum: 2026-10-28

## Kontekst in problem

Popravi.si ima tri vloge (stranka, izvajalec, administrator) in vsaka ima kar nekaj svojih zaslonov. Odločiti smo se morali, ali bo strežnik sam izrisoval strani ali pa bosta frontend in backend ločena in se bosta pogovarjala prek API.

## Obravnavane možnosti

* ločen frontend (React SPA) in REST API z JSON
* strani izrisuje strežnik (predloge v backendu, npr. Askama ali Tera)
* full-stack ogrodje, npr. Next.js, kjer sta frontend in backend skupaj

## Odločitev

Izbrana možnost: "ločen frontend in REST API", ker lahko tako frontend in backend razvijamo in testiramo vsakega posebej. Backend lahko testiramo samo prek API, brez brskalnika, frontend pa lahko delamo naprej, tudi če kakšen endpoint še ni končan. Zaslone, kot je primerjava ponudb ali sledenje statusu, je v Reactu lažje narediti kot s predlogami na strežniku. Next.js smo izpustili, ker bi potem backend pisali v JavaScriptu oziroma TypeScriptu, mi pa smo za backend izbrali Rust (glej [0002](0002-backend-v-rustu-z-axumom.md)). Isti API bi lahko kasneje uporabila tudi mobilna aplikacija.

### Posledice

* Dobro, ker je meja med frontendom in backendom jasna in jo lahko opišemo z OpenAPI (Swagger).
* Dobro, ker se oba dela lahko razvijata in testirata ločeno.
* Slabo, ker moramo urediti CORS in paziti, da se tipi na frontendu ujemajo z API.
* Slabo, ker sta to dva projekta z dvema buildoma namesto enega.
