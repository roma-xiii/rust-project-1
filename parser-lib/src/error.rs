use std::fmt;

/// Ошибки, возникающие при парсинге или сериализации банковских записей.
#[derive(Debug)]
pub enum ParseError {
  /// Ошибка ввода-вывода.
  InvalidIo(std::io::Error),
  /// Неверный магический заголовок в бинарном формате.
  InvalidMagic,
  /// Поле содержит недопустимое значение.
  InvalidField { field: &'static str, value: String },
  /// Обязательное поле отсутствует в записи.
  MissingField(&'static str),
  /// Данные не являются корректной строкой UTF-8.
  IsNotUtf8(std::str::Utf8Error),
}

/// Псевдоним результата с ошибкой [`ParseError`].
pub type Result<T> = std::result::Result<T, ParseError>;

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ParseError::InvalidIo(e) => write!(f, "IO error: {e}"),
      ParseError::InvalidMagic => write!(f, "invalid magic bytes in binary record header"),
      ParseError::InvalidField { field, value } => {
        write!(f, "invalid value for field '{field}': '{value}'")
      }
      ParseError::MissingField(field) => write!(f, "missing required field '{field}'"),
      ParseError::IsNotUtf8(e) => write!(f, "UTF-8 error: {e}"),
    }
  }
}

impl std::error::Error for ParseError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      ParseError::InvalidIo(e) => Some(e),
      ParseError::IsNotUtf8(e) => Some(e),
      _ => None,
    }
  }
}

impl From<std::io::Error> for ParseError {
  fn from(e: std::io::Error) -> Self {
    ParseError::InvalidIo(e)
  }
}

impl From<std::str::Utf8Error> for ParseError {
  fn from(e: std::str::Utf8Error) -> Self {
    ParseError::IsNotUtf8(e)
  }
}
