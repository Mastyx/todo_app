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
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

// -----------------------------------------------
pub fn run() -> io::Result<()> {
    // abilita il raw mode
    // la pressione dei tasti danno subito il comando senza aspettare
    // invio
    enable_raw_mode()?;

    // schermo alternativo
    let mut standard_out = io::stdout();
    execute!(standard_out, EnterAlternateScreen)?;

    // il terminale ha bisogno di un backend:
    // chi disegna i caratteri
    let backend = CrosstermBackend::new(standard_out);
    let mut terminal = Terminal::new(backend)?;

    // cereiamo un vettore di test con dei dati
    let elementi_prova = vec!["Task 1", "Task 2", "Task 3"];
    let mut selezionato = ListState::default();
    selezionato.select(Some(0));

    loop {
        terminal.draw(|f| {
            // dividiamo lo schermo in 2 diamo un layout
            let container = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(f.area());
            // -----------------------------------

            let item: Vec<ListItem> = elementi_prova.iter().map(|v| ListItem::new(*v)).collect();
            let lista = List::new(item)
                .block(Block::default().borders(Borders::ALL).title("Task"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">");

            // la lista ha bisogno di sapere quale riga e selezionata
            f.render_stateful_widget(lista, container[0], &mut selezionato);
            // container[1] parte inferiore
            let help = Paragraph::new("premi q per uscire").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Aiuto")
                    .border_type(BorderType::Rounded),
            );
            f.render_widget(help, container[1]);
            //
        })?;

        // intercettiamo gli eventi della tastiera
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => {
                    let i = selezionato
                        .selected()
                        .map_or(0, |i| (i + 1) % elementi_prova.len());
                    selezionato.select(Some(i));
                }
                KeyCode::Up => {
                    let i = selezionato.selected().map_or(0, |i| {
                        if i == 0 {
                            elementi_prova.len() - 1
                        } else {
                            i - 1
                        }
                    });
                    selezionato.select(Some(i));
                }
                _ => {}
            }
        }

        // --- settiamo la parte principale
    }
    // quando esce dal loop l'applicazione termina
    // fondamentale rispristinare il terminale
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
