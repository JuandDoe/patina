use std::{error::Error};
use std::thread;
use std::time::Duration;
use std::env;

use serde::Deserialize;
use serde_json::{json, Value};
use rusqlite::{params, Connection, Result};
use rusqlite::ToSql;
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

/// Un seul endroit qui sait parler à monero-wallet-rpc.
/// On envoie {method, params} et on récupère le champ "result" (ou une erreur).
fn rpc_call(method: &str, params: Value) -> Result<Value, Box<dyn Error>> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": method,
        "params": params,
    });

    let response: Value = ureq::post(RPC_URL).send_json(body)?.into_json()?;

    // Le wallet-rpc répond toujours en HTTP 200 : le succès OU l'erreur est DANS le JSON.
    if let Some(err) = response.get("error") {
        return Err(format!("Erreur RPC pour '{method}': {err}").into());
    }
    response
        .get("result")
        .cloned()
        .ok_or_else(|| format!("Réponse sans 'result' pour '{method}'").into())
}


fn main() -> Result<(), Box<dyn Error>> {
    // Collecter les arguments CLI passés au programme
    let args: Vec<String> = env::args().collect();
    let account_index: u64  = args[1].parse()?;

    // Montant attendu pour cette "facture". On le convertit en atomique une fois.
    let expected_xmr:f64 =  args[2].parse()?;

    let expected_atomic: u64 = (expected_xmr * ATOMIC_UNITS_PER_XMR as f64) as u64;

    let conn = Connection::open("sql/patina.db")?;
    db::create_invoices_table(&conn)?;
    db::save_invoice(&conn, "7BG5jr9QS5sGMdpbBrZEwVLZjSKJGJBsXdZLt8wiXyhhLjy7x2LZxsrAnHTgD8oG46ZtLjUGic2pWc96GFkGNPQQDA3Dt7Q", 0, 100_000_000_000, "pending")?;
    db::invoice_paid(&conn, "7BG5jr9QS5sGMdpbBrZEwVLZjSKJGJBsXdZLt8wiXyhhLjy7x2LZxsrAnHTgD8oG46ZtLjUGic2pWc96GFkGNPQQDA3Dt7Q")?;
    // 1) On génère une sous-adresse fraîche = la clé comptable de cette facture.
    let created = rpc_call(
        "create_address",
        json!({ "account_index": account_index, "label": "order-001" }),
    )?;

    let ca: CreatedAddress = serde_json::from_value(created)?;

    let address = ca.address;
    let address_index = ca.address_index;

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
        )?;
                
        let resp: GetTransfers = serde_json::from_value(transfers)?;

        let received: u64 = resp.incoming.iter().map(|t| t.amount).sum();
        let min_conf: Option<u64> = resp.incoming.iter().map(|t| t.confirmations).min();

        if received == 0 {
            println!("... rien pour l'instant, je revérifie dans {POLL_INTERVAL_SECS}s");
        } else if received < expected_atomic {
            let got = received as f64 / ATOMIC_UNITS_PER_XMR as f64;
            println!("Paiement partiel : {got} / {expected_xmr} XMR reçus");
        } else  {
                // On arrive ici => le MONTANT est atteint. Reste à savoir si c'est assez confirmé.
                let conf = min_conf.unwrap_or(0); // None = aucun transfert => on retient 0 (pas assez confirmé)
                if conf >= ULTIMATE_PART_CONFIRMATION {
                    let got = received as f64 / ATOMIC_UNITS_PER_XMR as f64;
                    println!("PAYÉ et confirmé ({conf} confirmations) : {got} XMR. Facture réglée.");
                    break;
                } else {
                    println!("Montant reçu, en attente de confirmations ({conf}/{ULTIMATE_PART_CONFIRMATION})");
                }
        }

        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }

    Ok(())
}

