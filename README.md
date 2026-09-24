# fleissner_cipher

Generatore di messaggi cifrati con la **griglia di Fleissner** (griglia rotante).
Dati una dimensione e un messaggio, crea una griglia cifrata e la maschera per
decifrarla, e salva la griglia in un SVG grafico pronto da pubblicare.

## Requisiti

- Rust stabile (edition 2021) con `cargo`.

## Uso

Eseguire **dalla root del repository** (template e storico usano percorsi relativi):

```sh
cargo run --release
```

```
=== BENVENUTO NEL GENERATORE FLEISSNER ===
Inserisci dimensione griglia (pari, min 4): 6
Inserisci il messaggio segreto: ciao mondo
```

La dimensione deve essere un numero pari `>= 4`. Gli spazi vengono ignorati;
un messaggio più corto di `n²` caratteri viene completato con lettere casuali,
uno più lungo viene troncato.

## Output

| Dove | Cosa |
|------|------|
| terminale | griglia cifrata, maschera (`X` = foro), istruzioni di decifratura |
| `fleissner_<n>_<slug>.svg` | griglia inserita nel template `assets/template.svg` |
| `database.json` | storico di tutte le esecuzioni |

`database.json` contiene griglia **e** maschera: chi lo legge ricostruisce
i messaggi. Non condividerlo e non committarlo.

## Decifrare

1. Sovrapporre la maschera alla griglia e leggere le lettere nei fori, da
   sinistra a destra e dall'alto in basso.
2. Ruotare la maschera di 90° in senso orario.
3. Ripetere fino a 4 letture.

## Struttura

```
src/main.rs           tutto il programma
assets/template.svg   template grafico con marcatore <!--FLEISSNER_GRID-->
prova_exp.svg         export SVG di riferimento (non usato dal codice)
docs/ARCHITETTURA.md  algoritmo, geometria SVG, formato storico, limitazioni
```

## Sviluppo

```sh
cargo test                                          # invarianti del cifrario
cargo doc --no-deps --document-private-items --open # documentazione del codice
```

Dettagli tecnici e limitazioni note: [docs/ARCHITETTURA.md](docs/ARCHITETTURA.md).
