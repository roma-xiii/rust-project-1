# ypbank-compare

CLI-утилита для сравнения файлов с банковскими транзакциями YPBank.

## Использование

```bash
ypbank_compare --file1 <файл> --format1 <bin|csv|txt> --file2 <файл> --format2 <bin|csv|txt>
```

## Примеры

```bash
# Сравнение bin и csv
cargo run -p ypbank-compare -- --file1 files/records_example.bin --format1 bin --file2 files/records_example.csv --format2 csv

# Сравнение txt и csv
cargo run -p ypbank-compare -- --file1 files/records_example.txt --format1 txt --file2 files/records_example.csv --format2 csv
```

## Вывод

- При совпадении: «Записи транзакций в 'file1' и 'file2' идентичны.»
- При различии: указывается индекс первой отличающейся транзакции и её содержимое в обоих файлах.
