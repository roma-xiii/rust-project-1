use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};

use crate::error::{ParseError, Result};
use crate::transaction::{Status, Transaction, TxType};

/// Парсит список транзакций из текстового формата.
///
/// Записи разделяются пустыми строками. Строки начинающиеся с `#` игнорируются.
/// Поля внутри записи могут располагаться в любом порядке.
pub fn parse<R: std::io::Read>(reader: &mut R) -> Result<Vec<Transaction>> {
  let reader = BufReader::new(reader);
  let mut transactions = Vec::new();
  let mut block: Vec<String> = Vec::new();

  for line in reader.lines() {
    let line = line?;
    let line = line.trim();

    if line.starts_with('#') {
      continue;
    }

    if line.is_empty() {
      if !block.is_empty() {
        transactions.push(parse_block(&block)?);
        block.clear();
      }
    } else {
      block.push(line.to_string());
    }
  }

  // Последний блок если файл не заканчивается пустой строкой
  if !block.is_empty() {
    transactions.push(parse_block(&block)?);
  }

  Ok(transactions)
}

/// Сериализует список транзакций в текстовый формат.
///
/// Записи разделяются пустой строкой.
pub fn serialize<W: Write>(records: &[Transaction], writer: &mut W) -> Result<()> {
  for (i, tx) in records.iter().enumerate() {
    if i > 0 {
      writeln!(writer)?;
    }
    writeln!(writer, "TX_ID: {}", tx.tx_id)?;
    writeln!(writer, "TX_TYPE: {}", tx_type_to_str(&tx.tx_type))?;
    writeln!(writer, "FROM_USER_ID: {}", tx.from_user_id)?;
    writeln!(writer, "TO_USER_ID: {}", tx.to_user_id)?;
    writeln!(writer, "AMOUNT: {}", tx.amount)?;
    writeln!(writer, "TIMESTAMP: {}", tx.timestamp)?;
    writeln!(writer, "STATUS: {}", status_to_str(&tx.status))?;
    writeln!(writer, "DESCRIPTION: \"{}\"", tx.description)?;
  }
  Ok(())
}

fn parse_block(lines: &[String]) -> Result<Transaction> {
  let mut map: HashMap<String, String> = HashMap::new();

  for line in lines {
    let (key, value) = parse_kv(line)?;
    map.insert(key.to_string(), value.to_string());
  }

  Ok(Transaction {
    tx_id: get_u64(&map, "TX_ID")?,
    tx_type: get_tx_type(&map)?,
    from_user_id: get_u64(&map, "FROM_USER_ID")?,
    to_user_id: get_u64(&map, "TO_USER_ID")?,
    amount: get_u64(&map, "AMOUNT")?,
    timestamp: get_u64(&map, "TIMESTAMP")?,
    status: get_status(&map)?,
    description: get_description(&map)?,
  })
}

fn parse_kv(line: &str) -> Result<(&str, &str)> {
  let pos = line.find(": ").ok_or_else(|| ParseError::InvalidField {
    field: "key-value pair",
    value: line.to_string(),
  })?;
  Ok((&line[..pos], line[pos + 2..].trim()))
}

fn get_u64(map: &HashMap<String, String>, field: &'static str) -> Result<u64> {
  let val = map.get(field).ok_or(ParseError::MissingField(field))?;
  val
    .trim()
    .parse::<u64>()
    .map_err(|_| ParseError::InvalidField {
      field,
      value: val.clone(),
    })
}

fn get_tx_type(map: &HashMap<String, String>) -> Result<TxType> {
  match map.get("TX_TYPE").map(String::as_str) {
    Some("DEPOSIT") => Ok(TxType::Deposit),
    Some("TRANSFER") => Ok(TxType::Transfer),
    Some("WITHDRAWAL") => Ok(TxType::Withdrawal),
    Some(other) => Err(ParseError::InvalidField {
      field: "TX_TYPE",
      value: other.to_string(),
    }),
    None => Err(ParseError::MissingField("TX_TYPE")),
  }
}

fn get_status(map: &HashMap<String, String>) -> Result<Status> {
  match map.get("STATUS").map(String::as_str) {
    Some("SUCCESS") => Ok(Status::Success),
    Some("FAILURE") => Ok(Status::Failure),
    Some("PENDING") => Ok(Status::Pending),
    Some(other) => Err(ParseError::InvalidField {
      field: "STATUS",
      value: other.to_string(),
    }),
    None => Err(ParseError::MissingField("STATUS")),
  }
}

fn get_description(map: &HashMap<String, String>) -> Result<String> {
  let val = map
    .get("DESCRIPTION")
    .ok_or(ParseError::MissingField("DESCRIPTION"))?;
  let s = val.trim();
  if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
    Ok(s[1..s.len() - 1].to_string())
  } else {
    Err(ParseError::InvalidField {
      field: "DESCRIPTION",
      value: s.to_string(),
    })
  }
}

fn tx_type_to_str(tx_type: &TxType) -> &'static str {
  match tx_type {
    TxType::Deposit => "DEPOSIT",
    TxType::Transfer => "TRANSFER",
    TxType::Withdrawal => "WITHDRAWAL",
  }
}

fn status_to_str(status: &Status) -> &'static str {
  match status {
    Status::Success => "SUCCESS",
    Status::Failure => "FAILURE",
    Status::Pending => "PENDING",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_transactions() -> Vec<Transaction> {
    vec![
      Transaction {
        tx_id: 1234567890123456,
        tx_type: TxType::Deposit,
        from_user_id: 0,
        to_user_id: 9876543210987654,
        amount: 10000,
        timestamp: 1633036800000,
        status: Status::Success,
        description: "Terminal deposit".to_string(),
      },
      Transaction {
        tx_id: 2312321321321321,
        tx_type: TxType::Transfer,
        from_user_id: 1231231231231231,
        to_user_id: 9876543210987654,
        amount: 1000,
        timestamp: 1633056800000,
        status: Status::Failure,
        description: "User transfer".to_string(),
      },
    ]
  }

  #[test]
  fn test_round_trip() -> Result<()> {
    let original = sample_transactions();

    let mut buf = Vec::new();
    serialize(&original, &mut buf)?;

    let parsed = parse(&mut buf.as_slice())?;
    assert_eq!(original, parsed);
    Ok(())
  }

  #[test]
  fn test_arbitrary_field_order() -> Result<()> {
    let data = b"\
TX_TYPE: DEPOSIT
AMOUNT: 10000
TX_ID: 1234567890123456
STATUS: SUCCESS
TO_USER_ID: 9876543210987654
FROM_USER_ID: 0
TIMESTAMP: 1633036800000
DESCRIPTION: \"Terminal deposit\"
";
    let parsed = parse(&mut data.as_ref())?;
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].tx_id, 1234567890123456);
    assert_eq!(parsed[0].amount, 10000);
    Ok(())
  }

  #[test]
  fn test_comments_ignored() -> Result<()> {
    let data = b"\
# This is a comment
TX_ID: 1
# Another comment
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 2
AMOUNT: 100
TIMESTAMP: 1000000
STATUS: SUCCESS
DESCRIPTION: \"desc\"
";
    let parsed = parse(&mut data.as_ref())?;
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].tx_id, 1);
    Ok(())
  }

  #[test]
  fn test_last_block_without_trailing_newline() -> Result<()> {
    let data = b"TX_ID: 1\nTX_TYPE: DEPOSIT\nFROM_USER_ID: 0\nTO_USER_ID: 2\nAMOUNT: 100\nTIMESTAMP: 1000000\nSTATUS: SUCCESS\nDESCRIPTION: \"desc\"";
    let parsed = parse(&mut data.as_ref())?;
    assert_eq!(parsed.len(), 1);
    Ok(())
  }

  #[test]
  fn test_missing_field_error() {
    let data = b"TX_ID: 1\nTX_TYPE: DEPOSIT\nFROM_USER_ID: 0\nTO_USER_ID: 2\nAMOUNT: 100\nTIMESTAMP: 1000000\nSTATUS: SUCCESS\n";
    let result = parse(&mut data.as_ref());
    assert!(matches!(
      result,
      Err(ParseError::MissingField("DESCRIPTION"))
    ));
  }
}
