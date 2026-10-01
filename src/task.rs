// modulo task per la crazione della struttura task

use uuid::Uuid as uuid;

use chrono::Local;
// per rendere Task serializzabile
use serde::{Deserialize, Serialize};
// Serialize scrive Deserialize legge

// la struttura base il task
// cioè il compito da svoglere
// con le sue proprieta id titolo contenuto completato

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub data: String,

    // data esecuzione
    pub data_esecuzione: String,

    pub titolo: String,
    pub contenuto: String,
    pub completato: bool,
}
// implementiamo i metodi
impl Task {
    pub fn new(titolo: String, contenuto: String, data_esecuzione: String) -> Self {
        Self {
            id: uuid::new_v4().to_string(),
            data: Local::now().format("%d-%m-%y").to_string(),
            titolo: titolo,
            contenuto: contenuto,
            data_esecuzione: data_esecuzione,
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
