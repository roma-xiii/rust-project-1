use std::io::{BufRead, BufReader, Write};

use crate::error::{ParseError, Result};
use crate::transaction::{Status, Transaction, TxType};

const HEADER: &str = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";

/// Парсит список транзакций из CSV-формата.
///
/// Первая строка должна быть заголовком. Пустые строки игнорируются.
pub fn parse<R: std::io::Read>(reader: &mut R) -> Result<Vec<Transaction>> {
  let reader = BufReader::new(reader);
  let mut lines = reader.lines();

  let header = lines
    .next()
    .ok_or(ParseError::MissingField("CSV header"))??;

  let header_invalid = header.trim() != HEADER;
  if header_invalid {
    return Err(ParseError::InvalidField {
      field: "header",
      value: header.to_string(),
    });
  }

  let mut transactions = Vec::new();
  for line in lines {
    let line = line?;
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    transactions.push(parse_line(line)?);
  }

  Ok(transactions)
}

/// Сериализует список транзакций в CSV-формат.
pub fn serialize<W: Write>(records: &[Transaction], writer: &mut W) -> Result<()> {
  writeln!(writer, "{HEADER}")?;
  for tx in records {
    writeln!(
      writer,
      "{},{},{},{},{},{},{},\"{}\"",
      tx.tx_id,
      tx_type_to_str(&tx.tx_type),
      tx.from_user_id,
      tx.to_user_id,
      tx.amount,
      tx.timestamp,
      status_to_str(&tx.status),
      tx.description,
    )?;
  }
  Ok(())
}

fn parse_line(line: &str) -> Result<Transaction> {
  let parts: Vec<&str> = line.splitn(8, ',').collect();
  if parts.len() != 8 {
    return Err(ParseError::InvalidField {
      field: "CSV record",
      value: line.to_string(),
    });
  }

  Ok(Transaction {
    tx_id: parse_u64(parts[0], "TX_ID")?,
    tx_type: parse_tx_type(parts[1])?,
    from_user_id: parse_u64(parts[2], "FROM_USER_ID")?,
    to_user_id: parse_u64(parts[3], "TO_USER_ID")?,
    amount: parse_u64(parts[4], "AMOUNT")?,
    timestamp: parse_u64(parts[5], "TIMESTAMP")?,
    status: parse_status(parts[6])?,
    description: parse_description(parts[7])?,
  })
}

fn parse_u64(s: &str, field: &'static str) -> Result<u64> {
  s.trim()
    .parse::<u64>()
    .map_err(|_| ParseError::InvalidField {
      field,
      value: s.to_string(),
    })
}

fn parse_tx_type(s: &str) -> Result<TxType> {
  match s.trim() {
    "DEPOSIT" => Ok(TxType::Deposit),
    "TRANSFER" => Ok(TxType::Transfer),
    "WITHDRAWAL" => Ok(TxType::Withdrawal),
    other => Err(ParseError::InvalidField {
      field: "TX_TYPE",
      value: other.to_string(),
    }),
  }
}

fn parse_status(s: &str) -> Result<Status> {
  match s.trim() {
    "SUCCESS" => Ok(Status::Success),
    "FAILURE" => Ok(Status::Failure),
    "PENDING" => Ok(Status::Pending),
    other => Err(ParseError::InvalidField {
      field: "STATUS",
      value: other.to_string(),
    }),
  }
}

fn parse_description(s: &str) -> Result<String> {
  let s = s.trim();
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
        tx_id: 1001,
        tx_type: TxType::Deposit,
        from_user_id: 0,
        to_user_id: 501,
        amount: 50000,
        timestamp: 1672531200000,
        status: Status::Success,
        description: "Initial account funding".to_string(),
      },
      Transaction {
        tx_id: 1002,
        tx_type: TxType::Transfer,
        from_user_id: 501,
        to_user_id: 502,
        amount: 15000,
        timestamp: 1672534800000,
        status: Status::Failure,
        description: "Payment for services, invoice #123".to_string(),
      },
      Transaction {
        tx_id: 1003,
        tx_type: TxType::Withdrawal,
        from_user_id: 502,
        to_user_id: 0,
        amount: 1000,
        timestamp: 1672538400000,
        status: Status::Pending,
        description: "ATM withdrawal".to_string(),
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
  fn test_description_with_commas() -> Result<()> {
    let original = vec![Transaction {
      tx_id: 1002,
      tx_type: TxType::Transfer,
      from_user_id: 501,
      to_user_id: 502,
      amount: 15000,
      timestamp: 1672534800000,
      status: Status::Failure,
      description: "Payment for services, invoice #123".to_string(),
    }];

    let mut buf = Vec::new();
    serialize(&original, &mut buf)?;

    let parsed = parse(&mut buf.as_slice())?;
    assert_eq!(original[0].description, parsed[0].description);
    Ok(())
  }

  #[test]
  fn test_invalid_header() {
    let data = b"WRONG_HEADER\n1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"desc\"\n";
    let result = parse(&mut data.as_ref());
    assert!(matches!(
      result,
      Err(ParseError::InvalidField {
        field: "header",
        ..
      })
    ));
  }

  #[test]
  fn test_empty_lines_ignored() -> Result<()> {
    let data =
      b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\n1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"desc\"\n\n";
    let parsed = parse(&mut data.as_ref())?;
    assert_eq!(parsed.len(), 1);
    Ok(())
  }
}
