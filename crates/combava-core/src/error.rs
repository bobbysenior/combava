use crate::output::Diagnostic;

/// Échec d'une transpilation : le document contient au moins une erreur.
#[derive(Debug, Clone, thiserror::Error)]
#[error("le document contient des erreurs")]
pub struct TranspileError {
    /// Erreurs **et** avertissements, triés par `span.start`. Au moins une erreur.
    pub diagnostics: Vec<Diagnostic>,
}
