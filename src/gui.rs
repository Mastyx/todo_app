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
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::Task_app;

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

    // crea un app per task app
    let mut app = Task_app::new();
    app.crea_task(
        "Titolo 1 ".to_string(),
        "Contenuto del titolo 1 ".to_string(),
    );
    app.crea_task(
        "Titolo 2".to_string(),
        "Contenuto del titolo 2 ".to_string(),
    );
    app.crea_task(
        "Titolo 3".to_string(),
        "Contenuto del titolo 3 ".to_string(),
    );

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

            let item: Vec<ListItem> = app
                .tasks
                .iter()
                .map(|task| {
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
                        .map_or(0, |i| (i + 1) % app.tasks.len());
                    selezionato.select(Some(i));
                }
                KeyCode::Up => {
                    let i = selezionato
                        .selected()
                        .map_or(0, |i| if i == 0 { app.tasks.len() - 1 } else { i - 1 });
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
