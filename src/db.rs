 use rusqlite::{params, Connection, Result};

 pub struct Invoice {
   pub id: i64,
    pub address: String,
    pub address_index: u64,
    pub expected_atomic: u64,
    pub status: String,
    pub created_at: String,
 }
 
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

pub fn get_invoice(conn: &Connection, address: &str) -> Result<Invoice> {
    conn.query_row(
        "SELECT id, address, address_index, expected_atomic, status, created_at 
        FROM invoices WHERE address = ?1",
        (address,),
        |row| {
            Ok(Invoice {
                id: row.get(0)?,
                address: row.get(1)?,
                address_index: row.get::<_, i64>(2)? as u64,
                expected_atomic: row.get::<_, i64>(3)? as u64,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_invoices_table() {
        let conn = Connection::open_in_memory().unwrap();
        create_invoices_table(&conn).unwrap();
    }

    #[test]
    fn test_invoice_paid() {
        let conn = Connection::open_in_memory().unwrap();
        create_invoices_table(&conn).unwrap();
        save_invoice(&conn, "test_address", 0, 100, "pending").unwrap();
        invoice_paid(&conn, "test_address").unwrap();
        let invoice = get_invoice(&conn, "test_address").unwrap();
        assert_eq!(invoice.status, "paid");
    }



    #[test]
    fn test_get_invoice() {
        let conn = Connection::open_in_memory().unwrap();
        create_invoices_table(&conn).unwrap();
        save_invoice(&conn, "test_address", 0, 100, "pending").unwrap();
        let invoice = get_invoice(&conn, "test_address").unwrap();
        assert_eq!(invoice.address, "test_address");
        assert_eq!(invoice.address_index, 0);
        assert_eq!(invoice.expected_atomic, 100);
        assert_eq!(invoice.status, "pending");
    }
}