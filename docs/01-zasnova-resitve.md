# 01 Zasnova rešitve

## Ime rešitve

Rešitev smo poimenovali **Popravi.si**.

## Namen in problem

Ko doma nekaj pokvari, recimo pušča pipa ali ne dela vtičnica, večina ljudi mojstra še vedno išče po znancih, Facebook skupinah ali pa kar kliče številke z oglasov. Vsakemu je treba težavo razložiti znova. Ponudbe pridejo po telefonu, SMS-u ali mailu, zato jih je težko primerjati. Ko se dogovoriš, pa nimaš nikjer zapisano, kaj je bilo dogovorjeno in v kakšnem stanju je delo. Prav tako težko preveriš, ali je mojster zanesljiv.

Tudi manjšim mojstrom (večinoma s.p.) to ni v korist. Nimajo spletne strani, oglaševanje jih stane, posle pa dobijo bolj ali manj naključno.

Popravi.si to poveže na enem mestu. Stranka enkrat opiše delo, mojstri iz pravega področja in kraja pošljejo ponudbe, stranka jih vidi skupaj in izbere eno. Potem se vidi, v kateri fazi je delo, na koncu pa stranka pusti oceno, ki pomaga naslednjim.

## Ciljni uporabniki

Stranke so ljudje, ki imajo stanovanje ali hišo (lahko tudi najemniki ali manjša pisarna) in rabijo nekoga za manjše delo. Na začetku ciljamo na Maribor z okolico. Večinoma niso tehnično podkovani, zato mora biti aplikacija preprosta.

Izvajalci so mojstri in manjša podjetja: vodovodarji, električarji, pleskarji, hišniki, čistilni servisi. Želijo dobiti nekaj lokalnih del na teden in imeti pregled nad svojimi ponudbami.

Administrator je nekdo od nas, ki skrbi za platformo. Preverja mojstre, ureja kategorije in posreduje, če pride do težav.

## Kako deluje

Najlažje je opisati na primeru.

Ana živi na Taboru in ji pušča kuhinjska pipa. V aplikaciji izbere kategorijo Vodovodna dela, na kratko opiše težavo, vpiše poštno številko in napiše, da ji najbolj ustreza naslednji teden popoldne. Povpraševanje je zdaj odprto (`OPEN`).

Marko je vodovodar iz Maribora in ga je administrator že prej preveril. Anino povpraševanje se mu pokaže med deli, ki ustrezajo njegovi kategoriji in območju. Pošlje ponudbo za 60 €, delo bo trajalo približno uro, prišel bi lahko v torek. Ponudbo pošljeta še dva vodovodarja, povpraševanje dobi status `QUOTED`.

Ana vidi vse tri ponudbe skupaj, s ceno, terminom, oceno mojstra in številom opravljenih del. Izbere Markovo. Ostali dve se avtomatsko zavrneta, povpraševanje pa preide v `ACCEPTED`.

Ko Marko pride, v aplikaciji označi, da je delo v teku (`IN_PROGRESS`), ko konča, pa da je zaključeno (`COMPLETED`). Vsaka sprememba se zapiše, zato se kasneje vidi, kdo jo je naredil in kdaj. Ana na koncu Marka oceni s petimi zvezdicami in napiše kratek komentar (`REVIEWED`). Markova povprečna ocena se posodobi.

Ta potek od povpraševanja do ocene je glavni del aplikacije. Natančna pravila, kdo lahko kaj naredi, so v `02-funkcionalnosti-in-obseg.md`.
