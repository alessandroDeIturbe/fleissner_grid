# Architettura di fleissner_cipher

Tutto il programma sta in `src/main.rs`. Il flusso di un'esecuzione:

```
stdin (dimensione, messaggio)
  └─ Fleissner::new ──► griglia + maschera
       ├─ Fleissner::display ──────────────► stdout
       ├─ render_from_template
       │    └─ generate_grid_markup ───────► fleissner_<n>_<slug>.svg
       └─ append_history ──────────────────► database.json
```

## 1. Il cifrario

### Coordinate

Le celle sono `(riga, colonna)`, origine `(0, 0)` in alto a sinistra, `n` = lato.
La rotazione oraria di 90° è `(r, c) → (c, n-1-r)`.

### Preparazione del messaggio

1. Si rimuovono tutti i caratteri di spaziatura (`char::is_whitespace`).
2. Si applica `to_ascii_uppercase`: `a-z` diventano maiuscole; punteggiatura,
   cifre e lettere non ASCII (`è`, `ü`, …) restano invariate.
3. Se i caratteri sono meno di `n²`, si aggiungono lettere casuali `A..=Z`.
4. Se sono più di `n²`, quelli in eccesso vengono ignorati senza avviso.

### Generazione della maschera

Il quadrante in alto a sinistra ha `(n/2)²` celle. Ogni cella `(r, c)` del
quadrante individua un'*orbita* di 4 posizioni che si scambiano tra loro con la
rotazione:

| Rotazione | Posizione          |
|-----------|--------------------|
| 0°        | `(r, c)`           |
| 90°       | `(c, n-1-r)`       |
| 180°      | `(n-1-r, n-1-c)`   |
| 270°      | `(n-1-c, r)`       |

Per ogni orbita se ne sceglie una a caso: il foro. Si ottengono `n²/4` fori, e
ruotando la maschera 4 volte ogni cella della griglia finisce sotto un foro
esattamente una volta. Le maschere possibili sono `4^(n²/4)`: 262 144 per
`n = 6`, circa 4,3 miliardi per `n = 8`.

Con `n` dispari la cella centrale non appartiene a nessuna orbita; inoltre il
ciclo sul quadrante `⌊n/2⌋ × ⌊n/2⌋` lascerebbe vuote l'intera riga e colonna
centrali (`2n-1` celle). Per questo `main` accetta solo dimensioni pari `>= 4`.

### Cifratura

Per 4 volte:
1. ordina i fori correnti per riga, poi per colonna;
2. scrive i caratteri successivi del messaggio nelle celle dei fori;
3. ruota i fori di 90° in senso orario.

### Decifratura

Speculare: sovrapporre la maschera in posizione iniziale, leggere le lettere
visibili da sinistra a destra e dall'alto in basso, ruotare di 90° in senso
orario, ripetere per 4 volte. Si ottengono `n²` caratteri; quelli oltre la
lunghezza del messaggio sono riempimento casuale.

## 2. Output SVG

`render_from_template` legge `assets/template.svg` (≈12 MB: contiene immagini
incorporate) e sostituisce il commento `<!--FLEISSNER_GRID-->` con il gruppo
`<g id="fleissner-grid">` prodotto da `generate_grid_markup`. Il resto del
template (sfondo, mascotte, indovinello, handle Instagram) resta invariato.

La pagina ha `viewBox="0 0 2481 3508"` (A4 a 300 dpi). La griglia occupa sempre
lo stesso riquadro quadrato, qualunque sia `n`:

| Costante        | Valore    | Significato                                   |
|-----------------|-----------|-----------------------------------------------|
| `GRID_X0`       | 445.32    | x dell'angolo in alto a sinistra              |
| `GRID_Y0`       | 636.24    | y dell'angolo in alto a sinistra              |
| `GRID_SIDE`     | 1589.6    | lato del riquadro                             |
| `STROKE_W`      | 7.28      | spessore bordo celle (fisso)                  |
| `FONT_RATIO`    | 0.2982    | dimensione font / lato cella                  |
| `BASELINE_FRAC` | 0.61917   | baseline del testo, come frazione della cella |

Per ogni cella: lato `cell = GRID_SIDE / n`, rettangolo in
`(GRID_X0 + c·cell, GRID_Y0 + r·cell)`, testo centrato orizzontalmente a
`y + cell·BASELINE_FRAC` in Arial, `font-size = cell·FONT_RATIO`.

I valori sono ricavati componendo le matrici di trasformazione dell'export
Affinity originale (per `n = 8`). `prova_exp.svg` nella root è un export SVG
non referenziato dal codice.

Nome del file: `fleissner_<n>_<slug>.svg`, dove `slug` è formato dai primi 30
caratteri alfanumerici (Unicode, `char::is_alphanumeric`) del messaggio, con le
sole lettere ASCII portate in minuscolo (`senzatitolo` se vuoto). Lo slug rivela
parte del messaggio in chiaro.
Un file con lo stesso nome viene sovrascritto.

## 3. Storico (`database.json`)

Array JSON; ogni esecuzione aggiunge un oggetto:

```json
{
  "timestamp": 1790266763,
  "size": 6,
  "message_len": 9,
  "ciphertext": "CMRQGNKYHTLIAOMAQFONUKJCXDSEVOCESXXG",
  "mask": [[0, 0], [1, 5], [3, 0], [4, 5], [4, 1], [3, 1], [2, 0], [2, 1], [2, 2]],
  "svg_file": "fleissner_6_ciaomondo.svg"
}
```

| Campo         | Contenuto                                                 |
|---------------|-----------------------------------------------------------|
| `timestamp`   | secondi Unix                                              |
| `size`        | lato `n`                                                  |
| `message_len` | caratteri non-spazio del messaggio, prima del troncamento |
| `ciphertext`  | griglia letta per righe, `n²` caratteri                   |
| `mask`        | fori `[riga, colonna]` in posizione iniziale              |
| `svg_file`    | file SVG generato                                         |

**Sicurezza:** `ciphertext` + `mask` permettono di ricostruire il messaggio in
chiaro (applicando la decifratura e tenendo i primi `min(message_len, n²)`
caratteri). Il file equivale a un archivio dei messaggi: non va condiviso né
committato.

## 4. Invarianti verificate

Il modulo `tests` in `src/main.rs` verifica:

| Test | Invariante |
|------|------------|
| `maschera_copre_ogni_cella_una_volta_in_quattro_rotazioni` | `n²/4` fori, copertura esatta per `n` = 4, 6, 8, 10 |
| `decifratura_documentata_restituisce_il_messaggio` | la procedura della §1 restituisce il messaggio |
| `messaggio_troppo_lungo_viene_troncato` | troncamento a `n²` |
| `caratteri_non_ascii_non_vengono_convertiti` | `è` resta `è` |
| `markup_ha_una_cella_per_posizione` | `n²` `<rect>` e `n²` `<text>` |
| `template_contiene_il_marcatore_una_volta` | il template ha un solo marcatore |
| `caratteri_xml_vengono_escapati` (`#[ignore]`) | oggi fallisce: vedi limitazione 1 |

## 5. Limitazioni note

1. **Caratteri XML non escapati.** `<` o `&` nel messaggio producono un SVG non valido.
2. **Lettere non ASCII** restano nella griglia così come sono (anche minuscole).
3. **Troncamento silenzioso** se il messaggio supera `n²` caratteri non-spazio.
4. **Storico fragile.** Se `database.json` non è leggibile o non è deserializzabile come `Vec<HistoryEntry>` (JSON invalido, oppure schema diverso: campo mancante, modifica manuale, cambio della struct), viene sovrascritto con la sola nuova voce.
5. **Percorsi relativi.** `assets/template.svg` e `database.json` sono risolti rispetto alla directory corrente: eseguire dalla root del repository. Altrove la lettura del template va in panic prima di scrivere qualsiasi file; `database.json`, di per sé, verrebbe creato come storico separato nella directory corrente.
6. **Nessuna riproducibilità.** Il generatore è `thread_rng()` senza seed. `rand_chacha` è dichiarato in `Cargo.toml` ma non usato.
7. **Nessun limite superiore a `n`.** Con griglie grandi il bordo (spessore fisso 7.28) occupa una parte crescente della cella.
8. **SVG pesanti.** Ogni output copia il template da ≈12 MB.
