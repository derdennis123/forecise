//! Brier Score calculation for accuracy tracking.
//!
//! Brier Score = (1/N) * Σ(forecast_i - outcome_i)²
//! Lower is better. Range: 0 (perfect) to 1 (worst).

use anyhow::Result;
use bigdecimal::BigDecimal;
use std::str::FromStr;

/// Calculate the Brier Score for a single prediction.
/// - `predicted`: the probability assigned (0.0 to 1.0)
/// - `actual`: the outcome (0.0 or 1.0)
pub fn brier_score_single(predicted: f64, actual: f64) -> f64 {
    (predicted - actual).powi(2)
}

/// Calculate the average Brier Score for a set of predictions.
pub fn brier_score_average(predictions: &[(f64, f64)]) -> Option<f64> {
    if predictions.is_empty() {
        return None;
    }

    let sum: f64 = predictions.iter()
        .map(|(pred, actual)| brier_score_single(*pred, *actual))
        .sum();

    Some(sum / predictions.len() as f64)
}

/// Convert Brier Score to an accuracy percentage (0-100%).
/// Uses a calibrated transformation: accuracy = (1 - brier_score) * 100
/// A Brier Score of 0.25 (random guessing on binary) = 75% accuracy
pub fn brier_to_accuracy_pct(brier_score: f64) -> f64 {
    ((1.0 - brier_score) * 100.0).clamp(0.0, 100.0)
}

/// Calculate Brier Score as BigDecimal for database storage.
pub fn brier_score_decimal(predicted: &BigDecimal, actual: &BigDecimal) -> Result<BigDecimal> {
    let pred_f64: f64 = predicted.to_string().parse()?;
    let actual_f64: f64 = actual.to_string().parse()?;
    let score = brier_score_single(pred_f64, actual_f64);
    Ok(BigDecimal::from_str(&format!("{:.6}", score))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_prediction() {
        assert!((brier_score_single(1.0, 1.0) - 0.0).abs() < 1e-10);
        assert!((brier_score_single(0.0, 0.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_worst_prediction() {
        assert!((brier_score_single(1.0, 0.0) - 1.0).abs() < 1e-10);
        assert!((brier_score_single(0.0, 1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_moderate_prediction() {
        let score = brier_score_single(0.7, 1.0);
        assert!((score - 0.09).abs() < 1e-10);
    }

    #[test]
    fn test_average_brier() {
        let predictions = vec![
            (0.9, 1.0),  // good
            (0.1, 0.0),  // good
            (0.5, 1.0),  // mediocre
        ];
        let avg = brier_score_average(&predictions).unwrap();
        // (0.01 + 0.01 + 0.25) / 3 = 0.09
        assert!((avg - 0.09).abs() < 1e-10);
    }

    #[test]
    fn test_empty_predictions() {
        assert_eq!(brier_score_average(&[]), None);
    }

    #[test]
    fn test_accuracy_conversion() {
        assert!((brier_to_accuracy_pct(0.0) - 100.0).abs() < 1e-10);
        assert!((brier_to_accuracy_pct(0.25) - 75.0).abs() < 1e-10);
        assert!((brier_to_accuracy_pct(1.0) - 0.0).abs() < 1e-10);
    }
}
