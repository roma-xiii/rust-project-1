use std::io::{Read, Write};

use crate::error::{ParseError, Result};
use crate::transaction::{Status, Transaction, TxType};

const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];

/// Парсит список транзакций из бинарного формата YPBank.
///
/// Каждая запись начинается с магических байт `YPBN`, за которыми следует
/// размер тела и само тело. Все числа в формате big-endian.
pub fn parse<R: Read>(reader: &mut R) -> Result<Vec<Transaction>> {
  let mut transactions = Vec::new();

  loop {
    // Читаем первый байт отдельно чтобы отличить нормальный EOF от обрыва записи
    let mut first = [0u8; 1];
    match reader.read_exact(&mut first) {
      Ok(()) => {}
      Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
      Err(e) => return Err(e.into()),
    }

    let mut rest_magic = [0u8; 3];
    reader.read_exact(&mut rest_magic)?;

    if first[0] != MAGIC[0] || rest_magic != MAGIC[1..] {
      return Err(ParseError::InvalidMagic);
    }

    let mut size_buf = [0u8; 4];
    reader.read_exact(&mut size_buf)?;
    let record_size = u32::from_be_bytes(size_buf) as usize;

    let mut body = vec![0u8; record_size];
    reader.read_exact(&mut body)?;

    transactions.push(parse_body(&body)?);
  }

  Ok(transactions)
}

/// Сериализует список транзакций в бинарный формат YPBank.
///
/// Каждая запись предваряется магическим заголовком `YPBN` и размером тела.
pub fn serialize<W: Write>(records: &[Transaction], writer: &mut W) -> Result<()> {
  for tx in records {
    let body = build_body(tx);
    writer.write_all(&MAGIC)?;
    writer.write_all(&(body.len() as u32).to_be_bytes())?;
    writer.write_all(&body)?;
  }
  Ok(())
}

fn parse_body(body: &[u8]) -> Result<Transaction> {
  let mut pos = 0;

  let tx_id = read_u64(body, &mut pos, "TX_ID")?;
  let tx_type = read_tx_type(body, &mut pos)?;
  let from_user_id = read_u64(body, &mut pos, "FROM_USER_ID")?;
  let to_user_id = read_u64(body, &mut pos, "TO_USER_ID")?;
  let amount = read_amount(body, &mut pos)?;
  let timestamp = read_u64(body, &mut pos, "TIMESTAMP")?;
  let status = read_status(body, &mut pos)?;
  let desc_len = read_u32(body, &mut pos, "DESC_LEN")? as usize;
  let description = read_string(body, &mut pos, desc_len)?;

  Ok(Transaction {
    tx_id,
    tx_type,
    from_user_id,
    to_user_id,
    amount,
    timestamp,
    status,
    description,
  })
}

fn read_u64(body: &[u8], pos: &mut usize, field: &'static str) -> Result<u64> {
  let end = *pos + 8;
  let bytes: [u8; 8] = body
    .get(*pos..end)
    .and_then(|s| s.try_into().ok())
    .ok_or(ParseError::MissingField(field))?;
  *pos = end;
  Ok(u64::from_be_bytes(bytes))
}

fn read_u32(body: &[u8], pos: &mut usize, field: &'static str) -> Result<u32> {
  let end = *pos + 4;
  let bytes: [u8; 4] = body
    .get(*pos..end)
    .and_then(|s| s.try_into().ok())
    .ok_or(ParseError::MissingField(field))?;
  *pos = end;
  Ok(u32::from_be_bytes(bytes))
}

fn read_amount(body: &[u8], pos: &mut usize) -> Result<u64> {
  let end = *pos + 8;
  let bytes: [u8; 8] = body
    .get(*pos..end)
    .and_then(|s| s.try_into().ok())
    .ok_or(ParseError::MissingField("AMOUNT"))?;
  *pos = end;
  // Спецификация описывает поле как знаковое i64, но фактическая сумма всегда
  // неотрицательна — знак избыточен при наличии TX_TYPE.
  Ok(i64::from_be_bytes(bytes).unsigned_abs())
}

fn read_tx_type(body: &[u8], pos: &mut usize) -> Result<TxType> {
  let byte = *body.get(*pos).ok_or(ParseError::MissingField("TX_TYPE"))?;
  *pos += 1;
  match byte {
    0 => Ok(TxType::Deposit),
    1 => Ok(TxType::Transfer),
    2 => Ok(TxType::Withdrawal),
    other => Err(ParseError::InvalidField {
      field: "TX_TYPE",
      value: other.to_string(),
    }),
  }
}

fn read_status(body: &[u8], pos: &mut usize) -> Result<Status> {
  let byte = *body.get(*pos).ok_or(ParseError::MissingField("STATUS"))?;
  *pos += 1;
  match byte {
    0 => Ok(Status::Success),
    1 => Ok(Status::Failure),
    2 => Ok(Status::Pending),
    other => Err(ParseError::InvalidField {
      field: "STATUS",
      value: other.to_string(),
    }),
  }
}

fn read_string(body: &[u8], pos: &mut usize, len: usize) -> Result<String> {
  let end = *pos + len;
  let bytes = body
    .get(*pos..end)
    .ok_or(ParseError::MissingField("DESCRIPTION"))?;
  let s = std::str::from_utf8(bytes).map_err(ParseError::IsNotUtf8)?;
  *pos = end;
  let s = strip_description_quotes(s);
  Ok(s.to_string())
}

fn strip_description_quotes(s: &str) -> &str {
  let s = s.trim();
  if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
    &s[1..s.len() - 1]
  } else {
    s
  }
}

fn build_body(tx: &Transaction) -> Vec<u8> {
  let desc = tx.description.as_bytes();
  let mut body = Vec::with_capacity(46 + desc.len());

  body.extend_from_slice(&tx.tx_id.to_be_bytes());
  body.push(tx_type_byte(&tx.tx_type));
  body.extend_from_slice(&tx.from_user_id.to_be_bytes());
  body.extend_from_slice(&tx.to_user_id.to_be_bytes());
  body.extend_from_slice(&(tx.amount as i64).to_be_bytes());
  body.extend_from_slice(&tx.timestamp.to_be_bytes());
  body.push(status_byte(&tx.status));
  body.extend_from_slice(&(desc.len() as u32).to_be_bytes());
  body.extend_from_slice(desc);

  body
}

fn tx_type_byte(tx_type: &TxType) -> u8 {
  match tx_type {
    TxType::Deposit => 0,
    TxType::Transfer => 1,
    TxType::Withdrawal => 2,
  }
}

fn status_byte(status: &Status) -> u8 {
  match status {
    Status::Success => 0,
    Status::Failure => 1,
    Status::Pending => 2,
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
        description: "Initial funding".to_string(),
      },
      Transaction {
        tx_id: 1002,
        tx_type: TxType::Transfer,
        from_user_id: 501,
        to_user_id: 502,
        amount: 15000,
        timestamp: 1672534800000,
        status: Status::Failure,
        description: "Transfer".to_string(),
      },
      Transaction {
        tx_id: 1003,
        tx_type: TxType::Withdrawal,
        from_user_id: 502,
        to_user_id: 0,
        amount: 1000,
        timestamp: 1672538400000,
        status: Status::Pending,
        description: "ATM".to_string(),
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
  fn test_magic_bytes_present() -> Result<()> {
    let tx = sample_transactions();
    let mut buf = Vec::new();
    serialize(&tx, &mut buf)?;

    assert_eq!(&buf[0..4], &MAGIC);
    Ok(())
  }

  #[test]
  fn test_invalid_magic() {
    let data = b"\x00\x00\x00\x00\x00\x00\x00\x08garbage!";
    let result = parse(&mut data.as_ref());
    assert!(matches!(result, Err(ParseError::InvalidMagic)));
  }

  #[test]
  fn test_empty_description() -> Result<()> {
    let original = vec![Transaction {
      tx_id: 1,
      tx_type: TxType::Deposit,
      from_user_id: 0,
      to_user_id: 1,
      amount: 100,
      timestamp: 1000000,
      status: Status::Success,
      description: String::new(),
    }];

    let mut buf = Vec::new();
    serialize(&original, &mut buf)?;

    let parsed = parse(&mut buf.as_slice())?;
    assert_eq!(original, parsed);
    Ok(())
  }

  #[test]
  fn test_parse_example_file() -> Result<()> {
    let data = include_bytes!("../../files/records_example.bin");
    let records = parse(&mut data.as_ref())?;
    assert!(!records.is_empty());
    Ok(())
  }
}
