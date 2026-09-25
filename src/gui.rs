// interfaccia grafica (ratatui)
use std::io;

use crossterm::{
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
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::Task_app;
// DEcidiamo di dare degli stati all'applicazione
// attraverso un enum
enum Mode {
    Normal,
    InserisciTitolo,
    InserisciContenuto,
    ConfermaElimina(usize),
}

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

    // schermo alternativo
    let mut standard_out = io::stdout();
    execute!(standard_out, EnterAlternateScreen)?;

    // il terminale ha bisogno di un backend:
    // chi disegna i caratteri
    let backend = CrosstermBackend::new(standard_out);
    let mut terminal = Terminal::new(backend)?;

    // crea un istanza di app per task app
    // di esempio
    let mut app = Task_app::new();
    for i in 0..20 {
        app.crea_task(
            format!("Titolo  {}", i).to_string(),
            format!("Contenuto del titolo {} ", i).to_string(),
        );
    }
    // inizializza lo stato di tracciamento
    // per la lista selezionabile
    let mut selezionato = ListState::default();
    selezionato.select(Some(0));

    loop {
        // creiamo un vettore ordinato
        let mut order_vec: Vec<usize> = (0..app.tasks.len()).collect();
        order_vec.sort_by_key(|&i| app.tasks[i].completato);

        terminal.draw(|f| {
            // dividiamo lo schermo in 2 diamo un layout
            let container = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(f.area());
            // -----------------------------------
            // vettore contenente i task
            let item: Vec<ListItem> = order_vec
                .iter()
                .map(|&id| {
                    let task = &app.tasks[id];
                    let simbolo = if task.completato { "[x]" } else { "[ ]" };
                    let riga = format!("{} {} - {}", simbolo, task.titolo, task.contenuto);
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

            // spazio riservato alla lista dei task
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
            f.render_stateful_widget(lista, container[0], &mut selezionato);

            // indicatore di scroll
            // per segnalare che ci sono delle task non visibili
            // utilizziamo Rect che importiamo
            let totale = order_vec.len();
            let altezza_visibile = container[0].height.saturating_sub(2) as usize;
            let offset = selezionato.offset(); // indice del primo elemento mostrato
            // eventualita che ce altro sopra
            if offset > 0 {
                let area = Rect::new(
                    container[0].x + container[0].width.saturating_sub(2),
                    container[0].y,
                    1,
                    1,
                );
                f.render_widget(Paragraph::new("↑"), area);
            }
            // eventualita che ce altro sotto
            if offset + altezza_visibile < totale {
                let area = Rect::new(
                    container[0].x + container[0].width.saturating_sub(2),
                    container[0].y + container[0].height.saturating_sub(1),
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
            f.render_widget(input, container[1]);

            // ----------------------------
            // bARRA DI AIUTO
            // container[1] parte inferiore
            let help = match mode {
                Mode::Normal => {
                    "[q] esci | su/giu : naviga | [n] : new | [invio] : complete | [d] : canc"
                }
                Mode::InserisciTitolo => "scrivi il titolo, [invio] per continuare",
                Mode::InserisciContenuto => "inserisci il conetenuto, [invio] salva",
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
            f.render_widget(help_widget, container[2]);
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
                        let i = selezionato
                            .selected()
                            .map_or(0, |i| (i + 1) % app.tasks.len());
                        selezionato.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = selezionato
                            .selected()
                            .map_or(0, |i| if i == 0 { app.tasks.len() - 1 } else { i - 1 });
                        selezionato.select(Some(i));
                    }
                    // intercettiamo la pressione del tasto enter e spazio
                    // per cambiare lo stato del task
                    // dobbiamo cercare la poszione reale data da order_vec e
                    // altrimenti l'indice reale non coincide
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(display_i) = selezionato.selected() {
                            if let Some(&real_i) = order_vec.get(display_i) {
                                if let Some(task) = app.tasks.get(real_i) {
                                    let id = task.id.clone();
                                    app.change_status(&id);
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
                    KeyCode::Enter => {
                        let titolo = input_titolo.trim().to_string();
                        let contenuto = input_contenuto.trim().to_string();
                        if !titolo.is_empty() {
                            app.crea_task(titolo, contenuto);
                        }
                        mode = Mode::Normal;
                    }
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Char(c) => input_contenuto.push(c),
                    KeyCode::Backspace => {
                        input_contenuto.pop();
                    }
                    _ => {}
                },
                // la pressione del d si entra in Modalita ConfermaElimina
                // cioe la conferma tramite 'y' o il tasoto invio per cancellare il task
                Mode::ConfermaElimina(real_i) => match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        if real_i < app.tasks.len() {
                            app.tasks.remove(real_i);
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
