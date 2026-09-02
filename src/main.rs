#![allow(warnings)]

mod task;

use crate::task::Task;
#[derive(Debug)]
struct Task_app {
    // struttura che gestisce i task
    tasks: Vec<Task>,
}
impl Task_app {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    // aggiunge elementi al vettore
    fn crea_task(&mut self, titolo: String, contenuto: String) {
        let nuovo_task = Task::new(titolo, contenuto);
        self.tasks.push(nuovo_task);
    }
    fn task_list(&self) {
        // elenca i task in ordine di creazione
        for task in &self.tasks {
            println!("ID : {}", task.id);
            println!("Titolo : {}", task.titolo);
            println!("Testo : {}", task.contenuto);
            println!("----------------------------")
        }
    }
}

// ---------------------------------------------------------------
fn main() {
    let mut app = Task_app::new();

    app.crea_task("Task 1".to_string(), "Cose da fare nel task1".to_string());
    app.crea_task("Task 2".to_string(), "Cose da fare nel task2".to_string());
    app.crea_task("Task 3".to_string(), "Cose da fare nel task3".to_string());

    println!("{:?}", &app.tasks);
    println!("----------------");
    app.task_list();
}
