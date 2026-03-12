# ypbank-converter

CLI-утилита для конвертации банковских записей между форматами YPBank.

## Использование

```bash
ypbank_converter --input <файл> --input-format <bin|csv|txt> --output-format <bin|csv|txt>
```

Результат выводится в stdout.

## Примеры

```bash
# bin → csv
cargo run -p ypbank-converter -- --input files/records_example.bin --input-format bin --output-format csv > output.csv

# csv → txt
cargo run -p ypbank-converter -- --input files/records_example.csv --input-format csv --output-format txt

# txt → bin
cargo run -p ypbank-converter -- --input files/records_example.txt --input-format txt --output-format bin > output.bin
```
