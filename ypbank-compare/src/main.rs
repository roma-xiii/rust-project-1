use std::fs::File;
use std::io;

use clap::{Parser, ValueEnum};
use parser_lib::{bin_format, csv_format, transaction::Transaction, txt_format};

/// Утилита для сравнения файлов с транзакциями YPBank.
///
/// Читает данные о транзакциях из двух файлов в указанных форматах,
/// сравнивает их и сообщает о результате.
///
/// Пример:
///   ypbank_compare --file1 records_example.bin --format1 bin --file2 records_example.csv --format2 csv
#[derive(Parser)]
#[command(name = "ypbank_compare", version, about, long_about = None)]
struct Cli {
  /// Первый файл с транзакциями
  #[arg(long)]
  file1: std::path::PathBuf,

  /// Формат первого файла
  #[arg(long, value_name = "FORMAT")]
  format1: Format,

  /// Второй файл с транзакциями
  #[arg(long)]
  file2: std::path::PathBuf,

  /// Формат второго файла
  #[arg(long, value_name = "FORMAT")]
  format2: Format,
}

#[derive(Clone, ValueEnum)]
enum Format {
  /// Бинарный формат YPBankBin
  Bin,
  /// CSV-формат YPBankCsv
  Csv,
  /// Текстовый формат YPBankText
  Txt,
}

fn main() {
  let cli = Cli::parse();

  if let Err(e) = run(cli) {
    eprintln!("Ошибка: {e}");
    std::process::exit(1);
  }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
  let mut f1 = File::open(&cli.file1)
    .map_err(|e| format!("не удалось открыть '{}': {e}", cli.file1.display()))?;
  let mut f2 = File::open(&cli.file2)
    .map_err(|e| format!("не удалось открыть '{}': {e}", cli.file2.display()))?;

  let tx1 = parse_input(&cli.format1, &mut f1)?;
  let tx2 = parse_input(&cli.format2, &mut f2)?;

  match compare_transactions(&tx1, &tx2) {
    None => {
      println!(
        "Записи транзакций в '{}' и '{}' идентичны.",
        cli.file1.display(),
        cli.file2.display()
      );
      Ok(())
    }
    Some(diff) => {
      println!(
        "Записи транзакций в '{}' и '{}' различаются:\n{}",
        cli.file1.display(),
        cli.file2.display(),
        diff
      );
      // Различие транзакций считается успешным завершением с кодом 0
      // с точки зрения работы программы, поэтому не возвращаем ошибку.
      Ok(())
    }
  }
}

fn parse_input(
  format: &Format,
  reader: &mut impl io::Read,
) -> Result<Vec<Transaction>, parser_lib::ParseError> {
  match format {
    Format::Bin => bin_format::parse(reader),
    Format::Csv => csv_format::parse(reader),
    Format::Txt => txt_format::parse(reader),
  }
}

fn compare_transactions(a: &[Transaction], b: &[Transaction]) -> Option<String> {
  if a.len() != b.len() {
    return Some(format!(
      "Разное количество записей: {} и {}",
      a.len(),
      b.len()
    ));
  }

  for (idx, (t1, t2)) in a.iter().zip(b.iter()).enumerate() {
    if t1 != t2 {
      return Some(format!(
        "Первое отличие на индексе {} (tx_id {} vs {}):\n  Слева:  {:?}\n  Справа: {:?}",
        idx, t1.tx_id, t2.tx_id, t1, t2
      ));
    }
  }

  None
}
