//! Módulo de utilidades para operaciones sobre enteros con validación de errores,
//! logging asíncrono y evaluación perezosa mediante programación funcional.

use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// Representa los posibles errores que pueden surgir al validar un entero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvenError {
    NegativeNotAllowed(i32),
    ZeroIsAmbiguous,
}

impl fmt::Display for EvenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvenError::NegativeNotAllowed(n) => write!(f, "El número {} es negativo", n),
            EvenError::ZeroIsAmbiguous => write!(f, "El cero no se considera ni par ni impar"),
        }
    }
}

impl std::error::Error for EvenError {}

/// Wrapper funcional que encapsula un entero y permite encadenar operaciones.
#[derive(Clone)]
pub struct EvenChecker<T> {
    value: Option<i32>,
    logger: Arc<dyn Fn(&str) + Send + Sync>,
    _phantom: PhantomData<T>,
}

/// Tipo de estado para valores aún no evaluados.
pub struct Unevaluated;
/// Tipo de estado para valores ya validados.
pub struct Validated;

impl Default for EvenChecker<Unevaluated> {
    fn default() -> Self {
        Self {
            value: None,
            logger: Arc::new(|s| eprintln!("[LOG] {}", s)),
            _phantom: PhantomData,
        }
    }
}

impl EvenChecker<Unevaluated> {
    /// Constructor que recibe un valor y devuelve una instancia funcional.
    pub fn new(n: i32) -> Self {
        Self {
            value: Some(n),
            logger: Arc::new(|s| eprintln!("[LOG] {}", s)),
            _phantom: PhantomData,
        }
    }

    /// Cambia la función de log por una personalizada.
    pub fn with_logger<F>(mut self, logger: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.logger = Arc::new(logger);
        self
    }

    /// Valida el número y devuelve un `EvenChecker` en estado `Validated`.
    pub fn validate(self) -> Result<EvenChecker<Validated>, EvenError> {
        let n = self.value.ok_or(EvenError::ZeroIsAmbiguous)?;
        if n < 0 {
            (self.logger)(&format!("Intento con número negativo: {}", n));
            return Err(EvenError::NegativeNotAllowed(n));
        }
        if n == 0 {
            (self.logger)("Cero detectado; se considera ambiguo");
            return Err(EvenError::ZeroIsAmbiguous);
        }
        (self.logger)(&format!("Validado: {}", n));
        Ok(EvenChecker {
            value: self.value,
            logger: self.logger,
            _phantom: PhantomData,
        })
    }
}

impl EvenChecker<Validated> {
    /// Decide si el número es par utilizando lógica funcional.
    pub fn is_even(&self) -> bool {
        let n = self.value.unwrap();
        (self.logger)(&format!("Comprobando paridad de {}", n));
        n % 2 == 0
    }

    /// Devuelve una función cerrada que permite reutilizar la lógica.
    pub fn to_predicate(&self) -> impl Fn(i32) -> bool + '_ {
        move |x: i32| x == self.value.unwrap() && self.is_even()
    }
}

/// Función de alto nivel que combina todas las etapas.
pub fn check_even(n: i32) -> Result<bool, EvenError> {
    EvenChecker::new(n).validate().map(|v| v.is_even())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_even_success() {
        let result = check_even(42).unwrap();
        assert!(result);
    }

    #[test]
    fn test_odd_success() {
        let result = check_even(13).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_negative_rejected() {
        let err = check_even(-4).unwrap_err();
        assert_eq!(err, EvenError::NegativeNotAllowed(-4));
    }

    #[test]
    fn test_zero_rejected() {
        let err = check_even(0).unwrap_err();
        assert_eq!(err, EvenError::ZeroIsAmbiguous);
    }

    #[test]
    fn test_functional_chaining() {
        let logs: Arc<Mutex<Vec<String>>> = Default::default();
        let captured = logs.clone();

        let is_even = EvenChecker::new(100)
            .with_logger(move |s| captured.lock().unwrap().push(s.to_string()))
            .validate()
            .map(|v| v.to_predicate())
            .unwrap();

        assert!(is_even(100)); // mismo valor
        assert!(!is_even(101)); // distinto valor

        let entries = logs.lock().unwrap();
        assert!(entries.iter().any(|s| s.contains("Validado")));
    }
}