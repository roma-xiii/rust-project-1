/// Тип банковской транзакции.
#[derive(Debug, Clone, PartialEq)]
pub enum TxType {
  /// Пополнение счёта.
  Deposit,
  /// Перевод между счетами.
  Transfer,
  /// Списание со счёта.
  Withdrawal,
}

/// Статус выполнения транзакции.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
  /// Транзакция выполнена успешно.
  Success,
  /// Транзакция завершилась ошибкой.
  Failure,
  /// Транзакция ожидает обработки.
  Pending,
}

/// Банковская транзакция.
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
  /// Уникальный идентификатор транзакции.
  pub tx_id: u64,
  /// Тип транзакции.
  pub tx_type: TxType,
  /// Идентификатор счёта отправителя. `0` для [`TxType::Deposit`].
  pub from_user_id: u64,
  /// Идентификатор счёта получателя. `0` для [`TxType::Withdrawal`].
  pub to_user_id: u64,
  /// Сумма в наименьших единицах валюты (центах). Всегда неотрицательная.
  pub amount: u64,
  /// Время транзакции в миллисекундах от Unix-эпохи.
  pub timestamp: u64,
  /// Статус транзакции.
  pub status: Status,
  /// Текстовое описание транзакции.
  pub description: String,
}
