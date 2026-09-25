use std::{fs, io};

use crate::task::Task;
// gestisce tutto cio che fa l'applicazione
// tutte le operazioni sui task, creazione

const FILE_TASKS: &str = "tasks.json";

#[derive(Debug)]
pub struct Task_app {
    // GESTISCE I CONTENITORE DI TASK
    pub tasks: Vec<Task>,
}
impl Task_app {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    // AGGIUNGE ELEMENTI AL VETTORE
    pub fn crea_task(&mut self, titolo: String, contenuto: String) {
        let nuovo_task = Task::new(titolo, contenuto);
        self.tasks.push(nuovo_task);
    }

    pub fn task_list(&self) {
        // elenca i task in ordine di creazione
        for task in &self.tasks {
            println!("ID : {}", task.id);
            println!("Titolo : {}", task.titolo);
            println!("Testo : {}", task.contenuto);
            println!("completato : {}", task.completato);
            println!("----------------------------")
        }
    }
    pub fn active_list(&self) {
        // elenca i task attivi cioe quelli con il campo false
        println!(" - - Task non completati - - ");
        for task in &self.tasks {
            if !task.completato {
                println!("{}", task.id);
                println!("{}", task.titolo);
            }
        }
    }

    pub fn change_status(&mut self, id: &String) {
        // cambia lo stato del task

        // cerca il task per id
        let mut task_da_modificare = self.tasks.iter_mut().find(|task| task.get_id() == id);
        // ITER MU PERMETTE DI FARE DELLE MODIFICHE
        match task_da_modificare {
            Some(task) => {
                task.completato = !task.completato;
            }
            None => {}
        }
    }

    // salva il vettore dei task su file
    // in formato json
    pub fn salva_task(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.tasks)?;
        fs::write(FILE_TASKS, json)?;
        Ok(())
    }

    // carica il task dal file se esiste
    pub fn carica_task() -> Self {
        match fs::read_to_string(FILE_TASKS) {
            Ok(contenuto) => {
                match serde_json::from_str(&contenuto) {
                    Ok(tasks) => Self { tasks },
                    Err(_) => {
                        // file corrotto o vuoto
                        Self::new()
                    }
                }
            }
            Err(_) => Self::new(),
        }
    }
}
