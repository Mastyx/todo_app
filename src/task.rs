// modulo task per la crazione della struttura task

use uuid::Uuid as uuid;

// per rendere Task serializzabile
use serde::{Deserialize, Serialize};
// Serialize scrive Deserialize legge

// la struttura base il task
// cioè il compioto da svoglere
// con le suo proprieta

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub titolo: String,
    pub contenuto: String,
    pub completato: bool,
}
// implementiamo i metodi
impl Task {
    pub fn new(titolo: String, contenuto: String) -> Self {
        Self {
            id: uuid::new_v4().to_string(),
            titolo: titolo,
            contenuto: contenuto,
            completato: false,
        }
    }

    // ritorna l'id del task
    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn update_content(&mut self, new_content: String) {
        self.contenuto = new_content;
    }
}
