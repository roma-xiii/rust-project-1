# parser-lib

Библиотека для парсинга и сериализации банковских транзакций YPBank в трёх форматах.

## Форматы

| Модуль       | Формат    | Описание                                      |
|--------------|-----------|-----------------------------------------------|
| `bin_format` | YPBankBin | Бинарный: big-endian, магические байты `YPBN` |
| `csv_format` | YPBankCsv | CSV с заголовком `TX_ID,TX_TYPE,...`          |
| `txt_format` | YPBankText| Ключ: значение, записи разделены пустой строкой |

## Использование

Все парсеры принимают `Read` и `Write` — можно использовать файлы, stdin/stdout, буферы.

```rust
use parser_lib::{bin_format, csv_format, txt_format, transaction::Transaction};

// Чтение из файла
let mut file = std::fs::File::open("records.bin")?;
let transactions: Vec<Transaction> = bin_format::parse(&mut file)?;

// Запись в stdout
let mut stdout = std::io::stdout();
csv_format::serialize(&transactions, &mut stdout)?;
```

## Зависимости

Стандартная библиотека. Внешних зависимостей нет.
