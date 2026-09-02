// modulo task per la crazione della struttura task
use uuid::Uuid as uuid;

#[derive(Debug)]
pub struct Task {
    pub id: String,
    pub titolo: String,
    pub contenuto: String,
    pub completato: bool,
}

impl Task {
    pub fn new(titolo: String, contenuto: String) -> Self {
        Self {
            id: uuid::new_v4().to_string(),
            titolo: titolo,
            contenuto: contenuto,
            completato: false,
        }
    }
}
