use std::{error::Error};
use std::time::Duration;
use std::env;

use serde::Deserialize;
use serde_json::{json, Value};
use rusqlite::{Connection, Result};
mod db;

// --- Configuration du v0 (on la "durcira" plus tard via fichier/env) ---
const RPC_URL: &str = "http://127.0.0.1:38083/json_rpc";
const POLL_INTERVAL_SECS: u64 = 5;
// 1 XMR = 1_000_000_000_000 unités atomiques ("piconero").
// On manipule TOUJOURS des entiers atomiques pour l'argent (jamais de float en interne).
const ATOMIC_UNITS_PER_XMR: u64 = 1_000_000_000_000;

//Nombre de confirmation minimale pour considérer un paiement comme effectué
const ULTIMATE_PART_CONFIRMATION:u64 = 1;

// On peuple les variables directement avec Serde. parse not validate 
    #[derive(Deserialize)]
    struct CreatedAddress {
         address: String,
          address_index: u64,
    }

    #[derive(Deserialize)]
        struct Transfer {
        amount: u64,
        confirmations: u64,
    }

    #[derive(Deserialize)]
        struct GetTransfers {
            #[serde(rename = "in", default)]
            incoming: Vec<Transfer>
        }

        #[derive(Debug, PartialEq)]
enum PaymentStatus {
    Nothing,
    Partial,
    AwaitingConfirmations,
    Confirmed,
}

fn evaluate_payment(received: u64, expected: u64, min_conf: Option<u64>, required_conf: u64) -> PaymentStatus {
    // exactement ta logique du if/else, mais chaque branche fait `return ...` / renvoie un PaymentStatus
    // au lieu de println!
    if received == 0 {
        PaymentStatus::Nothing
    } else if received < expected {
        PaymentStatus::Partial
    } else {
        let conf = min_conf.unwrap_or(0);
        if conf >= required_conf {
            PaymentStatus::Confirmed
        } else {
            PaymentStatus::AwaitingConfirmations
        }
    }
    
}

/// Un seul endroit qui sait parler à monero-wallet-rpc.
/// On envoie {method, params} et on récupère le champ "result" (ou une erreur).
/// 
/// 
/// 
/// let client = reqwest::Client::new();
   
async fn rpc_call(method: &str, params: Value) -> Result<Value, Box<dyn Error>> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": method,
        "params": params,
    });
    let client = reqwest::Client::new();
      let response: Value = client
         .post(RPC_URL)
         .json(&body)          // sérialise body en JSON (équivaut à send_json)
         .send()               // envoie — c'est de l'attente réseau
         .await?               // ... donc .await ici
         .json()               // décode la réponse en JSON
         .await?;              // ... encore de l'attente, donc .await


    // Le wallet-rpc répond toujours en HTTP 200 : le succès OU l'erreur est DANS le JSON.
    if let Some(err) = response.get("error") {
        return Err(format!("Erreur RPC pour '{method}': {err}").into());
    }
    response
        .get("result")
        .cloned()
        .ok_or_else(|| format!("Réponse sans 'result' pour '{method}'").into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Collecter les arguments CLI passés au programme
    let args: Vec<String> = env::args().collect();
    let account_index: u64  = args[1].parse()?;

    // Montant attendu pour cette "facture". On le convertit en atomique une fois.
    let expected_xmr:f64 =  args[2].parse()?;

    let expected_atomic: u64 = (expected_xmr * ATOMIC_UNITS_PER_XMR as f64) as u64;

    let conn = Connection::open("patina.db")?;
    db::create_invoices_table(&conn)?;
    
    // 1) On génère une sous-adresse fraîche = la clé comptable de cette facture.
    let created = rpc_call(
        "create_address",
        json!({ "account_index": account_index, "label": "order-001" }),
    ).await?;

    let ca: CreatedAddress = serde_json::from_value(created)?;

    let address = ca.address;
    let address_index = ca.address_index;

    db::save_invoice(&conn, &address, address_index, expected_atomic, "pending")?;

    println!("=== Nouvelle facture ===");
    println!("Envoie {expected_xmr} XMR à :");
    println!("  {address}");
    println!("  (sous-adresse index {address_index})");
    println!("En attente du paiement...\n");

    // 2) Boucle : on regarde ce qui a été reçu SUR CETTE sous-adresse uniquement.
    loop {
        let transfers = rpc_call(
            "get_transfers",
            json!({
                "in": true,
                "account_index": account_index,
                "subaddr_indices": [address_index],
            }),
        ).await?;
                
        let resp: GetTransfers = serde_json::from_value(transfers)?;

        let received: u64 = resp.incoming.iter().map(|t| t.amount).sum();
        let min_conf: Option<u64> = resp.incoming.iter().map(|t| t.confirmations).min();

        match evaluate_payment(received, expected_atomic, min_conf, ULTIMATE_PART_CONFIRMATION) {
            PaymentStatus::Nothing => {
                println!("... rien pour l'instant, je revérifie dans {POLL_INTERVAL_SECS}s");
            }
            PaymentStatus::Partial => {
                let got = received as f64 / ATOMIC_UNITS_PER_XMR as f64;
                println!("Paiement partiel : {got} / {expected_xmr} XMR reçus");
            }
            PaymentStatus::AwaitingConfirmations => {
                let conf = min_conf.unwrap_or(0);
                println!("Montant reçu, en attente de confirmations ({conf}/{ULTIMATE_PART_CONFIRMATION})");
            }
            PaymentStatus::Confirmed => {
                let got = received as f64 / ATOMIC_UNITS_PER_XMR as f64;
                db::invoice_paid(&conn, &address)?;
                println!("PAYÉ et confirmé : {got} XMR. Facture réglée.");
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_zero_received() {
        assert_eq!(evaluate_payment(0, 100, None, 2), PaymentStatus::Nothing);
    }

    #[test]
    fn if_less_than_expected() {
        assert_eq!(evaluate_payment(50, 100, None, 2), PaymentStatus::Partial);
    }

    #[test]
    fn if_enough_received_but_not_enough_confirmed() {
        assert_eq!(evaluate_payment(100, 100, Some(1), 2), PaymentStatus::AwaitingConfirmations);
    }

    #[test]
    fn if_enough_received_and_enough_confirmed() {
        assert_eq!(evaluate_payment(100, 100, Some(2), 2), PaymentStatus::Confirmed);
    }

    #[test]
    fn if_too_much_received_and_enough_confirmed() {
        assert_eq!(evaluate_payment(150, 100, Some(2), 2), PaymentStatus::Confirmed);
    }

}
