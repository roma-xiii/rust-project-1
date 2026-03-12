//! Cross-format integration tests: проверка конвертации bin↔csv, bin↔txt, csv↔txt.

use parser_lib::{Status, TxType, bin_format, csv_format, transaction::Transaction, txt_format};

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
fn test_bin_to_csv_roundtrip() -> parser_lib::Result<()> {
  let original = sample_transactions();

  let mut buf = Vec::new();
  bin_format::serialize(&original, &mut buf)?;
  let from_bin = bin_format::parse(&mut buf.as_slice())?;

  let mut buf2 = Vec::new();
  csv_format::serialize(&from_bin, &mut buf2)?;
  let from_csv = csv_format::parse(&mut buf2.as_slice())?;

  assert_eq!(original, from_csv);
  Ok(())
}

#[test]
fn test_bin_to_txt_roundtrip() -> parser_lib::Result<()> {
  let original = sample_transactions();

  let mut buf = Vec::new();
  bin_format::serialize(&original, &mut buf)?;
  let from_bin = bin_format::parse(&mut buf.as_slice())?;

  let mut buf2 = Vec::new();
  txt_format::serialize(&from_bin, &mut buf2)?;
  let from_txt = txt_format::parse(&mut buf2.as_slice())?;

  assert_eq!(original, from_txt);
  Ok(())
}

#[test]
fn test_csv_to_txt_roundtrip() -> parser_lib::Result<()> {
  let original = sample_transactions();

  let mut buf = Vec::new();
  csv_format::serialize(&original, &mut buf)?;
  let from_csv = csv_format::parse(&mut buf.as_slice())?;

  let mut buf2 = Vec::new();
  txt_format::serialize(&from_csv, &mut buf2)?;
  let from_txt = txt_format::parse(&mut buf2.as_slice())?;

  assert_eq!(original, from_txt);
  Ok(())
}
