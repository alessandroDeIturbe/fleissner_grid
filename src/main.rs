use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const HISTORY_PATH: &str = "database.json";

#[derive(Serialize, Deserialize)]
struct HistoryEntry {
    timestamp: u64,
    size: usize,
    message_len: usize,
    ciphertext: String,
    mask: Vec<(usize, usize)>,
    svg_file: String,
}

/// Legge lo storico esistente (se presente), aggiunge l'entry e riscrive il file.
fn append_history(entry: HistoryEntry) -> io::Result<()> {
    let mut history: Vec<HistoryEntry> = fs::read_to_string(HISTORY_PATH)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default();

    history.push(entry);

    let json = serde_json::to_string_pretty(&history).expect("serializzazione storico fallita");
    fs::write(HISTORY_PATH, json)
}

struct Fleissner {
    size: usize,
    grid: Vec<Vec<char>>,
    mask: Vec<(usize, usize)>,
}

impl Fleissner {
    /// Crea una nuova griglia basata sulla dimensione e sul messaggio forniti dall'utente.
    fn new(size: usize, message: &str) -> Self {
        let mut rng = rand::thread_rng();
        let total_cells = size * size;
        let mut grid = vec![vec![' '; size]; size];
        
        // Pulizia messaggio: togliamo spazi e rendiamo tutto maiuscolo
        let mut chars: Vec<char> = message
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_uppercase())
            .collect();

        // Riempimento con lettere casuali se il messaggio è troppo corto
        while chars.len() < total_cells {
            chars.push(rng.gen_range(b'A'..=b'Z') as char);
        }

        // 1. Generazione della maschera (1/4 della dimensione totale)
        // Usiamo la logica delle "orbite" rotazionali per evitare sovrapposizioni
        let mut mask_indices = Vec::new();
        let half = size / 2;
        for r in 0..half {
            for c in 0..half {
                let positions = [
                    (r, c),                         // 0°
                    (c, size - 1 - r),             // 90°
                    (size - 1 - r, size - 1 - c),   // 180°
                    (size - 1 - c, r),             // 270°
                ];
                // Scegliamo casualmente uno dei 4 fori possibili per questa orbita
                let chosen = positions.choose(&mut rng).unwrap();
                mask_indices.push(*chosen);
            }
        }

        // 2. Scrittura del messaggio nella griglia tramite rotazioni della maschera
        let mut current_mask = mask_indices.clone();
        let mut msg_idx = 0;

        for _ in 0..4 {
            // Ordina la maschera corrente (opzionale, per scrivere da sinistra a destra)
            current_mask.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
            
            for &(r, c) in &current_mask {
                if msg_idx < total_cells {
                    grid[r][c] = chars[msg_idx];
                    msg_idx += 1;
                }
            }
            // Ruota le coordinate della maschera per il prossimo quadrante di messaggio
            current_mask = current_mask.iter()
                .map(|&(r, c)| (c, size - 1 - r))
                .collect();
        }

        Fleissner { size, grid, mask: mask_indices }
    }

    /// Stampa a video la griglia finale e la maschera di decifrazione
    fn display(&self) {
        let line = "=".repeat(self.size * 2 + 5);
        
        println!("\n{}", line);
        println!("  GRIGLIA CIFRATA (Il messaggio da inviare)");
        println!("{}", line);
        for row in &self.grid {
            print!("  ");
            for &ch in row { print!("{} ", ch); }
            println!();
        }

        println!("\n{}", line);
        println!("  MASCHERA DI DECIFRAZIONE (I buchi)");
        println!("{}", line);
        let mut mask_visual = vec![vec!['.'; self.size]; self.size];
        for &(r, c) in &self.mask { mask_visual[r][c] = 'X'; }
        for row in mask_visual {
            print!("  ");
            for ch in row { print!("{} ", ch); }
            println!();
        }
        
        println!("\nIstruzioni:");
        println!("1. Applica la maschera sulla griglia e leggi le lettere segnate con X.");
        println!("2. Ruota la maschera di 90° in senso ORARIO.");
        println!("3. Ripeti fino a completare i 4 giri.\n");
    }

}

const TEMPLATE_PATH: &str = "assets/template.svg";
const GRID_MARKER: &str = "<!--FLEISSNER_GRID-->";

// Geometria ricavata componendo le matrici Affinity del template originale
// (matrix(1.017278,...) ∘ matrix(0.686987,...) ∘ matrix(0.636092,...) per i
// riquadri, matrix(0.983015,...) per le lettere): il riquadro griglia occupa
// sempre lo stesso spazio fisso nella pagina, qualunque sia `size`.
const GRID_X0: f64 = 445.32;
const GRID_Y0: f64 = 636.24;
const GRID_SIDE: f64 = 1589.6;
const STROKE_W: f64 = 7.28; // stroke-width finale nel template originale (size=8)
const FONT_RATIO: f64 = 0.2982; // font-size / dimensione cella
const BASELINE_FRAC: f64 = 0.61917; // posizione baseline del testo nella cella

/// Genera il markup SVG della sola griglia cifrata (nessuna maschera),
/// nello stile del template: quadrati bianchi bordo nero senza spazi tra
/// loro, lettere Arial centrate, dimensionati per adattarsi a `size`
/// mantenendo il riquadro complessivo fisso come nel template originale.
fn generate_grid_markup(cipher: &Fleissner) -> String {
    let cell = GRID_SIDE / cipher.size as f64;
    let font_size = cell * FONT_RATIO;

    let mut markup = String::from("<g id=\"fleissner-grid\">\n");
    for r in 0..cipher.size {
        for c in 0..cipher.size {
            let x = GRID_X0 + c as f64 * cell;
            let y = GRID_Y0 + r as f64 * cell;
            let cx = x + cell / 2.0;
            let ty = y + cell * BASELINE_FRAC;
            let ch = cipher.grid[r][c];
            write!(
                markup,
                r#"<rect x="{x}" y="{y}" width="{cell}" height="{cell}" style="fill:white;stroke:black;stroke-width:{STROKE_W}px;"/>
<text x="{cx}" y="{ty}" text-anchor="middle" style="font-family:'ArialMT', 'Arial', sans-serif;font-size:{font_size}px;">{ch}</text>
"#
            )
            .unwrap();
        }
    }
    markup.push_str("</g>");
    markup
}

/// Legge il template brandizzato e sostituisce il marcatore della griglia
/// con quella generata per questo `cipher`. Il resto del template (sfondo,
/// mascotte, indovinello, handle Instagram) resta invariato.
fn render_from_template(cipher: &Fleissner) -> String {
    let template = fs::read_to_string(TEMPLATE_PATH).expect("impossibile leggere il template SVG");
    template.replace(GRID_MARKER, &generate_grid_markup(cipher))
}

fn main() {
    let mut input = String::new();

    println!("=== BENVENUTO NEL GENERATORE FLEISSNER ===");

    // Input Dimensione
    let size: usize = loop {
        print!("Inserisci dimensione griglia (pari, min 4): ");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<usize>() {
            Ok(n) if n >= 4 && n % 2 == 0 => break n,
            _ => println!("Per favore, inserisci un numero pari maggiore o uguale a 4."),
        }
    };

    // Input Messaggio
    print!("Inserisci il messaggio segreto: ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let message = input.trim();

    // Esecuzione
    let cipher = Fleissner::new(size, message);
    cipher.display();

    let slug: String = message
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .take(30)
        .collect();
    let slug = if slug.is_empty() { "senzatitolo".to_string() } else { slug };

    let svg_path = format!("fleissner_{}_{}.svg", size, slug);
    fs::write(&svg_path, render_from_template(&cipher)).expect("impossibile scrivere il file SVG");
    println!("SVG generato: {}", svg_path);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let entry = HistoryEntry {
        timestamp,
        size,
        message_len: message.chars().filter(|c| !c.is_whitespace()).count(),
        ciphertext: cipher.grid.iter().flatten().collect(),
        mask: cipher.mask.clone(),
        svg_file: svg_path,
    };
    append_history(entry).expect("impossibile aggiornare database.json");
    println!("Storico aggiornato in {}", HISTORY_PATH);
}
