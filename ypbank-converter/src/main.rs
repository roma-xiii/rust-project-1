use std::fs::File;
use std::io::{self, BufWriter};

use clap::{Parser, ValueEnum};
use parser_lib::{bin_format, csv_format, transaction::Transaction, txt_format};

/// Конвертер банковских записей между форматами YPBank.
///
/// Читает транзакции из файла в заданном формате и выводит результат
/// в другом формате в stdout.
///
/// Пример:
///   ypbank_converter --input records.bin --input-format bin --output-format csv
#[derive(Parser)]
#[command(name = "ypbank_converter", version, about, long_about = None)]
struct Cli {
  /// Путь к входному файлу
  #[arg(long)]
  input: std::path::PathBuf,

  /// Формат входного файла
  #[arg(long, value_name = "FORMAT")]
  input_format: Format,

  /// Формат выходных данных (stdout)
  #[arg(long, value_name = "FORMAT")]
  output_format: Format,
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
    eprintln!("Error: {e}");
    std::process::exit(1);
  }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
  let mut file =
    File::open(&cli.input).map_err(|e| format!("cannot open '{}': {e}", cli.input.display()))?;

  let transactions = parse_input(&cli.input_format, &mut file)?;

  let stdout = io::stdout();
  let mut writer = BufWriter::new(stdout.lock());

  serialize_output(&cli.output_format, &transactions, &mut writer)?;

  Ok(())
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

fn serialize_output(
  format: &Format,
  transactions: &[Transaction],
  writer: &mut impl io::Write,
) -> Result<(), parser_lib::ParseError> {
  match format {
    Format::Bin => bin_format::serialize(transactions, writer),
    Format::Csv => csv_format::serialize(transactions, writer),
    Format::Txt => txt_format::serialize(transactions, writer),
  }
}
