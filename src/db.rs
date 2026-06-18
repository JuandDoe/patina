 use rusqlite::{params, Connection, Result};
 
 pub fn create_invoices_table(conn: &Connection) -> Result<()> {

    conn.execute(
        "CREATE TABLE IF NOT EXISTS invoices (
            id    INTEGER PRIMARY KEY,
            address  TEXT,
            address_index INTEGER,
            expected_atomic INTEGER,
            status TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP)",
            (),
    )?;
    Ok(()) //retour de la fonction
}

pub fn save_invoice(conn: &Connection, address: &str, address_index: u64, expected_atomic: u64, status: &str) -> Result<()> {
   conn.execute(
    "INSERT INTO invoices (address, address_index, expected_atomic, status) VALUES (?1, ?2, ?3, ?4)",
    (address, address_index as i64, expected_atomic as i64, status),
)?;
    Ok(())
}

pub fn invoice_paid(conn: &Connection, address: &str) -> Result<()> {
    conn.execute(
        "UPDATE invoices SET status = 'paid' WHERE address = ?1",
        (address,),
    )?;
    Ok(())
}