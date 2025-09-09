use std::fmt;
use std::sync::Arc;

/// Posibles errores de validación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    NegativeNotAllowed(i32),
    ZeroIsAmbiguous,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::NegativeNotAllowed(n) => write!(f, "El número {} es negativo", n),
            ValidationError::ZeroIsAmbiguous => write!(f, "El cero no se considera ni par ni impar"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Logger asíncrono: tipos de salida y un sender.
pub mod logger {
    use super::*;
    use std::io::Write;
    use std::sync::mpsc::{channel, Sender};
    use std::thread;

    pub trait LogWriter: Send + 'static {
        fn write(&mut self, entry: &str);
    }

    pub struct StderrWriter;
    impl LogWriter for StderrWriter {
        fn write(&mut self, entry: &str) {
            let _ = writeln!(std::io::stderr(), "{}", entry);
        }
    }

    pub struct ChannelWriter(pub Sender<String>);
    impl LogWriter for ChannelWriter {
        fn write(&mut self, entry: &str) {
            let _ = self.0.send(entry.to_string());
        }
    }

    /// Ejecuta el logger en un hilo separado; devuelve el sender y un handle.
    pub fn threaded<W: LogWriter>(mut writer: W) -> Sender<String> {
        let (tx, rx) = channel::<String>();
        thread::spawn(move || {
            while let Ok(line) = rx.recv() {
                writer.write(&line);
            }
        });
        tx
    }
}

/// Validador genérico sobre enteros.
pub struct Validator {
    logger: Arc<dyn Fn(&str) + Send + Sync>,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            logger: Arc::new(|s| {
                let _ = writeln!(std::io::stderr(), "{}", s);
            }),
        }
    }
}

impl Validator {
    /// Inyecta un logger personalizado.
    pub fn with_logger<L>(logger: L) -> Self
    where
        L: Fn(&str) + Send + Sync + 'static,
    {
        Self {
            logger: Arc::new(logger),
        }
    }

    /// Valida y devuelve el número si pasa las reglas.
    pub fn validate(&self, n: i32) -> Result<i32, ValidationError> {
        if n < 0 {
            (self.logger)(&format!("Intento con número negativo: {}", n));
            return Err(ValidationError::NegativeNotAllowed(n));
        }
        if n == 0 {
            (self.logger)("Cero detectado; se considera ambiguo");
            return Err(ValidationError::ZeroIsAmbiguous);
        }
        (self.logger)(&format!("Validado: {}", n));
        Ok(n)
    }

    /// Comprueba si un número validado es par.
    pub fn is_even(&self, n: i32) -> bool {
        (self.logger)(&format!("Comprobando paridad de {}", n));
        n % 2 == 0
    }

    /// Pipeline completo: valida y decide paridad.
    pub fn check_even(&self, n: i32) -> Result<bool, ValidationError> {
        self.validate(n).map(|m| self.is_even(m))
    }
}

/// Alias cómodo para el pipeline por defecto.
pub fn check_even(n: i32) -> Result<bool, ValidationError> {
    Validator::default().check_even(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn even_ok() {
        assert!(check_even(42).unwrap());
    }

    #[test]
    fn odd_ok() {
        assert!(!check_even(13).unwrap());
    }

    #[test]
    fn negative_fails() {
        let err = check_even(-10).unwrap_err();
        assert_eq!(err, ValidationError::NegativeNotAllowed(-10));
    }

    #[test]
    fn zero_fails() {
        let err = check_even(0).unwrap_err();
        assert_eq!(err, ValidationError::ZeroIsAmbiguous);
    }

    #[test]
    fn custom_logger() {
        let logs: Arc<Mutex<Vec<String>>> = Default::default();
        let captured = logs.clone();
        let validator = Validator::with_logger(move |s| {
            captured.lock().unwrap().push(s.to_string());
        });

        let _ = validator.check_even(100);
        let entries = logs.lock().unwrap();
        assert!(entries.iter().any(|s| s.contains("Validado")));
    }
}

/// Ejemplo de uso desde binario.
#[cfg(doctest)]
/// ```
/// use rainy_utils::check_even;
/// assert!(check_even(42).unwrap());
/// ```
pub struct ReadmeDoctests;
