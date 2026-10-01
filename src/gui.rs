// interfaccia grafica (ratatui)
use std::{collections, io, ops::ControlFlow};

// per i giorni della settimana
use chrono::{Datelike, Local, Months, NaiveDate, Weekday};
use std::collections::BTreeSet;

use crossterm::{
    //Command,
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

// ++ per implementare un editor esterno che facilita la scrittura del contenuto
use std::env;
use std::fs;
use std::process::Command;

use crate::app::Task_app;
// DEcidiamo di dare degli stati all'applicazione
// attraverso un enum
enum Mode {
    Normal,
    InserisciTitolo,
    InserisciContenuto,
    // scelta della data di esecuzione (selettore a frecce)
    InserisciData,
    ConfermaElimina(usize),
}

// nome del giorno della settimana in italiano
fn giorno_it(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "lunedi",
        Weekday::Tue => "martedi",
        Weekday::Wed => "mercoledi",
        Weekday::Thu => "giovedi",
        Weekday::Fri => "venerdi",
        Weekday::Sat => "sabato",
        Weekday::Sun => "domenica",
    }
}

// ++ aggiunta per editor esterno
//sospende la Tui, apre l'editor esterno su un file temporaneo
// contenente il testo passato e alla chiusura ritorna il nuovo testo
fn modifica_in_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    contenuto_attuale: &str,
) -> io::Result<String> {
    // file temporaneo dove scriviamo il contenuto da modificare
    let percorso_tmp = env::temp_dir().join("todo_app_edit.txt");
    fs::write(&percorso_tmp, contenuto_attuale)?;

    // usciamo dalla modalita alternati per non andare in conflitto
    // lasciando il terminale libero all'editor
    disable_raw_mode();
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    // scegliamo l'editor : Variabile ambiente Editor altrimenti nano o vim

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    //lancia il processo e aspettiamo che l'utente chiuda
    let stato = Command::new(&editor).arg(&percorso_tmp).status();

    // qualunque cosa succeda si rientra nella Tui
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;

    // propaghiamo l'eventuale errore di lancio solo dopo aver ripristinato il terminale
    stato?;

    // rileggiamo il contenuto modificato
    let nuovo_contenuto = fs::read_to_string(&percorso_tmp)?;
    let _ = fs::remove_file(&percorso_tmp); // puliamo 

    Ok(nuovo_contenuto.trim_end().to_string())
}
// fine modifica editor //

// -----------------------------------------------
pub fn run() -> io::Result<()> {
    // abilita il raw mode
    // la pressione dei tasti danno subito il comando senza aspettare
    // invio
    enable_raw_mode()?;
    // cra un istanza dell enum
    // e gli da uno stato iniziale
    let mut mode = Mode::Normal;
    // le variabili che contengo i dati
    let mut input_titolo = String::new();
    let mut input_contenuto = String::new();
    // data di esecuzione scelta durante la creazione (default = oggi)
    let mut input_data: NaiveDate = Local::now().date_naive();

    // schermo alternativo
    let mut standard_out = io::stdout();
    execute!(standard_out, EnterAlternateScreen)?;

    // il terminale ha bisogno di un backend:
    // chi disegna i caratteri
    let backend = CrosstermBackend::new(standard_out);
    let mut terminal = Terminal::new(backend)?;

    // crea un istanza di app per task app
    // di esempio
    let mut app = Task_app::carica_task();
    // inizializza lo stato di tracciamento
    // per la lista selezionabile
    let mut selezionato = ListState::default();
    selezionato.select(Some(0));
    // indice del giorno selezionato nella barra superiore
    // None = nessun filtro, la lista mostra tutti i task
    let mut giorno_idx: Option<usize> = None;

    loop {
        // creiamo un vettore ordinato
        let mut order_vec: Vec<usize> = (0..app.tasks.len()).rev().collect();
        // prima i non completati, poi per data di esecuzione crescente
        // (sort stabile: a parita' di data resta l'ordine "ultimo creato prima")
        order_vec.sort_by_key(|&i| {
            let d = NaiveDate::parse_from_str(&app.tasks[i].data_esecuzione, "%d-%m-%y")
                .unwrap_or(NaiveDate::MAX);
            (app.tasks[i].completato, d)
        });

        // giorni di esecuzione che hanno almeno un task non completato, in ordine cronologico
        let giorni_vec: Vec<NaiveDate> = app
            .tasks
            .iter()
            .filter(|t| !t.completato)
            .filter_map(|t| NaiveDate::parse_from_str(&t.data_esecuzione, "%d-%m-%y").ok())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        // se il giorno selezionato non esiste piu' (es. l'ultimo task di
        // quel giorno e' stato completato) torniamo all'ultimo disponibile
        if let Some(idx) = giorno_idx {
            if giorni_vec.is_empty() {
                giorno_idx = None;
            } else if idx >= giorni_vec.len() {
                giorno_idx = Some(giorni_vec.len() - 1);
            }
        }

        // se e' selezionato un giorno, la lista mostra solo i task di quella data
        if let Some(idx) = giorno_idx {
            if let Some(giorno) = giorni_vec.get(idx) {
                order_vec.retain(|&i| {
                    NaiveDate::parse_from_str(&app.tasks[i].data_esecuzione, "%d-%m-%y")
                        .map(|d| d == *giorno)
                        .unwrap_or(false)
                });
            }
        }

        // la riga selezionata deve restare dentro i limiti dopo il filtro
        if order_vec.is_empty() {
            selezionato.select(None);
        } else if let Some(i) = selezionato.selected() {
            let max = order_vec.len() - 1;
            if i > max {
                selezionato.select(Some(max));
            }
        } else {
            selezionato.select(Some(0));
        }

        terminal.draw(|f| {
            // dividiamo lo schermo in 2 diamo un layout
            let container = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // parte superiore: giorni che hanno almeno un task non completato
            // giorni_vec e giorno_idx sono calcolati fuori dalla draw
            let oggi_data = Local::now().date_naive();
            let mut spans: Vec<Span> = Vec::new();
            for (i, giorno) in giorni_vec.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  |  "));
                }
                let e_selezionato = giorno_idx == Some(i);
                let stile = if e_selezionato {
                    // il giorno scelto con le frecce ha lo sfondo evidenziato
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else if *giorno == oggi_data {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(giorno.format("%d-%m-%y").to_string(), stile));
            }
            let giorni = Paragraph::new(Line::from(spans)).block(
                Block::default()
                    .title("Day  (←/→ filtra, esc mostra tutti)")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );
            f.render_widget(giorni, container[0]);

            // dividiamo il container[1] la parte alta piu grande
            // in 2 colonne affiancate
            let colonne = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(container[1]);

            // -----------------------------------
            // vettore contenente i task
            let item: Vec<ListItem> = order_vec
                .iter()
                .map(|&id| {
                    let task = &app.tasks[id];
                    let simbolo = if task.completato { "[x]" } else { "[ ]" };
                    let riga = format!("{} {}", simbolo, task.titolo);
                    let stile = if task.completato {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(riga, stile)))
                })
                .collect();

            // spazio riservato alla lista di sinistra
            // colonne[0] visualizza solo il segno e il titolo
            let lista = List::new(item)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Task")
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">");
            // la lista ha bisogno  di sapere quale riga e selezionata
            f.render_stateful_widget(lista, colonne[0], &mut selezionato);

            // colonne[1]-  colonna a destra per la visualizzazione del
            // contenuto in anteprima del titolo selezionato a sisnist
            //
            // memorizziamo il il contenuto in una variabile
            let contenuto_preview = if let Some(display_i) = selezionato.selected() {
                if let Some(&real_i) = order_vec.get(display_i) {
                    app.tasks.get(real_i).map(|t| {
                        if t.data_esecuzione.is_empty() {
                            t.contenuto.clone()
                        } else {
                            format!("Creato : {}\nDa eseguire in data : {}\n\n{}", t.data, t.data_esecuzione, t.contenuto)
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            };
            // nel caso non abbiamo selezionato nulla
            let testo_destra =
                contenuto_preview.unwrap_or_else(|| "Nessun task selezionato".to_string());
            // creiamo la parte destra
            let anteprima = Paragraph::new(testo_destra)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Contenuto")
                        .border_type(BorderType::Rounded),
                )
                .wrap(Wrap { trim: false });
            f.render_widget(anteprima, colonne[1]);

            // indicatore di scroll
            // per segnalare che ci sono delle task non visibili
            // utilizziamo Rect che importiamo
            let totale = order_vec.len();
            let altezza_visibile = colonne[0].height.saturating_sub(2) as usize;
            let offset = selezionato.offset(); // indice del primo elemento mostrato
            // eventualita che ce altro sopra
            if offset > 0 {
                let area = Rect::new(
                    colonne[0].x + colonne[0].width.saturating_sub(2),
                    colonne[0].y,
                    1,
                    1,
                );
                f.render_widget(Paragraph::new("↑"), area);
            }
            // eventualita che ce altro sotto
            if offset + altezza_visibile < totale {
                let area = Rect::new(
                    colonne[0].x + colonne[0].width.saturating_sub(2),
                    colonne[0].y + colonne[0].height.saturating_sub(1),
                    1,
                    1,
                );
                f.render_widget(Paragraph::new("↓"), area);
            }

            // ---------------------------------
            // riga di input visibile solo mentre si scrive
            let mut input_text = match mode {
                Mode::InserisciTitolo => format!("Titolo : {}_", input_titolo),
                Mode::InserisciContenuto => format!(
                    "Titolo  : {} | Contenuto : {}_",
                    input_titolo, input_contenuto
                ),
                Mode::InserisciData => format!(
                    "Titolo : {} | Data esecuzione : < {} > ({})",
                    input_titolo,
                    input_data.format("%d-%m-%y"),
                    giorno_it(input_data.weekday())
                ),
                Mode::Normal => String::new(),
                Mode::ConfermaElimina(real_i) => {
                    if let Some(task) = app.tasks.get(real_i) {
                        format!("Eliminare : {} ?", task.titolo)
                    } else {
                        String::new()
                    }
                }
            };
            let input = Paragraph::new(input_text)
                .block(Block::default().borders(Borders::ALL).title("new task"));
            f.render_widget(input, container[2]);

            // ----------------------------
            // bARRA DI AIUTO
            // container[1] parte inferiore
            let help = match mode {
                Mode::Normal => {
                    "[q] esci | su/giu : naviga | sx/dx : filtra giorno | [esc] : tutti | [n] : new | [d] : canc"
                }
                Mode::InserisciTitolo => "scrivi il titolo, [invio] per continuare",
                Mode::InserisciContenuto => "inserisci il conetenuto, [invio] per scegliere la data",
                Mode::InserisciData => {
                    "sx/dx : -/+ 1 giorno | su/giu : +/- 1 settimana | [pgsu]/[pggiu] : +/- 1 mese | [t] oggi | [invio] salva"
                }
                Mode::ConfermaElimina(_) => {
                    "[y] o [invio] per confermare | [n] o [esc] per annullare"
                }
            };
            let help_widget = Paragraph::new(help).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Aiuto")
                    .border_type(BorderType::Rounded),
            );
            f.render_widget(help_widget, container[3]);
            //
        })?;
        // finisce la parte grafica p
        // --------------------------------------

        // Gestione eventi ----------------------
        // intercettiamo gli eventi della tastieria
        if let Event::Key(key) = event::read()? {
            match mode {
                // nella modalita notrmale
                Mode::Normal => match key.code {
                    // esce dal loop dell'applicazione e termina
                    KeyCode::Char('q') => break,
                    // movimento tasti freccia
                    KeyCode::Down => {
                        // inseriamo il controllo if altrimenti andremo a fare
                        // una divisione per 0 andando in panic
                        if !order_vec.is_empty() {
                            let i = selezionato
                                .selected()
                                .map_or(0, |i| (i + 1) % order_vec.len());
                            selezionato.select(Some(i));
                        }
                    }
                    KeyCode::Up => {
                        if !order_vec.is_empty() {
                            let i = selezionato
                                .selected()
                                .map_or(0, |i| if i == 0 { order_vec.len() - 1 } else { i - 1 });
                            selezionato.select(Some(i));
                        }
                    }
                    // naviga tra i giorni della barra superiore; la lista
                    // dei task a sinistra si filtra in base al giorno scelto
                    KeyCode::Left => {
                        if !giorni_vec.is_empty() {
                            let nuovo = match giorno_idx {
                                None => giorni_vec.len() - 1,
                                Some(0) => giorni_vec.len() - 1,
                                Some(i) => i - 1,
                            };
                            giorno_idx = Some(nuovo);
                            selezionato.select(Some(0));
                        }
                    }
                    KeyCode::Right => {
                        if !giorni_vec.is_empty() {
                            let nuovo = match giorno_idx {
                                None => 0,
                                Some(i) if i + 1 >= giorni_vec.len() => 0,
                                Some(i) => i + 1,
                            };
                            giorno_idx = Some(nuovo);
                            selezionato.select(Some(0));
                        }
                    }
                    // esc toglie il filtro e rimostra tutti i task
                    KeyCode::Esc => {
                        giorno_idx = None;
                        selezionato.select(Some(0));
                    }
                    // intercettiamo la pressione del tasto enter e spazio
                    // per cambiare lo stato del task
                    // dobbiamo cercare la poszione reale data da order_vec e
                    // altrimenti l'indice reale non coincide
                    KeyCode::Char(' ') => {
                        if let Some(display_i) = selezionato.selected() {
                            if let Some(&real_i) = order_vec.get(display_i) {
                                if let Some(task) = app.tasks.get(real_i) {
                                    let id = task.id.clone();
                                    app.change_status(&id);
                                    let _ = app.salva_task();
                                }
                            }
                        }
                    }

                    // aple l'editor esterno sul contenuto del task selezionato
                    KeyCode::Enter => {
                        if let Some(display_i) = selezionato.selected() {
                            if let Some(&real_i) = order_vec.get(display_i) {
                                if let Some(task) = app.tasks.get(real_i) {
                                    let contenuto_attuale = task.contenuto.clone();
                                    match modifica_in_editor(&mut terminal, &contenuto_attuale) {
                                        Ok(nuovo_contenuto) => {
                                            if let Some(task_mut) = app.tasks.get_mut(real_i) {
                                                task_mut.update_content(nuovo_contenuto);
                                            }
                                            let _ = app.salva_task();
                                        }
                                        Err(_) => {}
                                    }
                                }
                            }
                        }
                    }
                    // n per creare una nuovo task
                    KeyCode::Char('n') => {
                        mode = Mode::InserisciTitolo;
                        input_titolo.clear();
                        input_contenuto.clear();
                    }

                    // d = delete per eliminare i task dalla lista
                    // anche qui come nel evento enter
                    // andiamo a nell order_vec che da l'id reale del task
                    KeyCode::Char('d') => {
                        if let Some(display_i) = selezionato.selected() {
                            if let Some(&real_i) = order_vec.get(display_i) {
                                mode = Mode::ConfermaElimina(real_i);
                            }
                        }
                    }
                    _ => {}
                },
                //  Quando siamo nella modalita Inserisci titolo
                // intercettiamo l'enter per passare al inserisci contenuto
                // esc per andare in normal mode
                // backspace per cancellare carattere per carattere
                // char invece intercetta il carattere premuto e l'aggiunge
                Mode::InserisciTitolo => match key.code {
                    KeyCode::Enter => mode = Mode::InserisciContenuto,
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Backspace => {
                        input_titolo.pop();
                    }
                    KeyCode::Char(c) => input_titolo.push(c),
                    _ => {}
                },
                // Quando siamo in modalita InserisciContenuto
                // Enter per completare e creare un nuovo task tramite crea_task
                // Backspace cancella input_contenuto
                // scrivendo intercettimo il tasto premuto con char che aggiunge a input_contenuto
                // Esc torna alla modalita Normal
                Mode::InserisciContenuto => match key.code {
                    // enter passa alla scelta della data di esecuzione
                    KeyCode::Enter => {
                        input_data = Local::now().date_naive();
                        mode = Mode::InserisciData;
                    }
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Char(c) => input_contenuto.push(c),
                    KeyCode::Backspace => {
                        input_contenuto.pop();
                    }
                    _ => {}
                },
                // Selezione della data di esecuzione con le frecce
                // Enter crea il task, Esc annulla
                Mode::InserisciData => match key.code {
                    KeyCode::Right => {
                        input_data = input_data.succ_opt().unwrap_or(input_data);
                    }
                    KeyCode::Left => {
                        input_data = input_data.pred_opt().unwrap_or(input_data);
                    }
                    KeyCode::Up => {
                        input_data = input_data
                            .checked_add_signed(chrono::Duration::days(7))
                            .unwrap_or(input_data);
                    }
                    KeyCode::Down => {
                        input_data = input_data
                            .checked_sub_signed(chrono::Duration::days(7))
                            .unwrap_or(input_data);
                    }
                    KeyCode::PageUp => {
                        input_data = input_data
                            .checked_add_months(Months::new(1))
                            .unwrap_or(input_data);
                    }
                    KeyCode::PageDown => {
                        input_data = input_data
                            .checked_sub_months(Months::new(1))
                            .unwrap_or(input_data);
                    }
                    KeyCode::Char('t') => input_data = Local::now().date_naive(),
                    KeyCode::Enter => {
                        let titolo = input_titolo.trim().to_string();
                        let contenuto = input_contenuto.trim().to_string();
                        if !titolo.is_empty() {
                            let data_esecuzione = input_data.format("%d-%m-%y").to_string();
                            app.crea_task(titolo, contenuto, data_esecuzione);
                            let _ = app.salva_task();
                        }
                        mode = Mode::Normal;
                    }
                    KeyCode::Esc => mode = Mode::Normal,
                    _ => {}
                },
                // la pressione del d si entra in Modalita ConfermaElimina
                // cioe la conferma tramite 'y' o il tasoto invio per cancellare il task
                Mode::ConfermaElimina(real_i) => match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        if real_i < app.tasks.len() {
                            app.tasks.remove(real_i);
                            let _ = app.salva_task();
                            if app.tasks.is_empty() {
                                selezionato.select(None);
                            } else if let Some(display_i) = selezionato.selected() {
                                let max = app.tasks.len() - 1;
                                if display_i > max {
                                    selezionato.select(Some(max));
                                }
                            }
                        }
                        mode = Mode::Normal;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => mode = Mode::Normal,
                    _ => {}
                },
            }
        }
    }
    // quando esce dal loop l'applicazione termina
    // fondamentale rispristinare il terminale
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
