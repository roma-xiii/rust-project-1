# Проект 1. Парсинг финансовых данных YPBank

Библиотека для парсинга, сериализации и десериализации банковских транзакций в трёх форматах, а также CLI-утилиты для конвертации и сравнения файлов.

## Структура проекта

```
project-1/
├── parser-lib/      # библиотека парсеров
├── ypbank-converter # конвертер форматов
├── ypbank-compare   # сравнение двух файлов
├── files/           # примеры файлов
└── docs/            # спецификации форматов
```

## Поддерживаемые форматы

- **YPBankBin** — бинарный формат (big-endian, магические байты `YPBN`)
- **YPBankCsv** — CSV-таблица с заголовком
- **YPBankText** — текстовый формат (ключ: значение, разделитель пустая строка)

## Сборка

```bash
cargo build --release
```

## CLI-утилиты

### ypbank-converter

Читает файл в заданном формате и выводит результат в другой формат в stdout.

```bash
cargo run -p ypbank-converter -- --input records.bin --input-format bin --output-format csv > output.csv
cargo run -p ypbank-converter -- --input records.csv --input-format csv --output-format txt
```

Возможные форматы: `bin`, `csv`, `txt`.

### ypbank-compare

Сравнивает транзакции из двух файлов (форматы могут различаться).

```bash
cargo run -p ypbank-compare -- --file1 records_example.bin --format1 bin --file2 records_example.csv --format2 csv
```

## Тесты

```bash
cargo test -p parser-lib
```

## Документация

Спецификации форматов:
- [`bin`](docs/YPBankBinFormat_ru.md)
- [`csv`](docs/YPBankCsvFormat_ru.md)
- [`txt`](docs/YPBankTextFormat_ru.md)
