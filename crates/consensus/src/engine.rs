//! Forecise Consensus Engine
//!
//! Calculates an accuracy-weighted consensus forecast from multiple sources.
//! The key differentiator: sources that have been historically more accurate
//! get higher weight in the consensus.

use anyhow::Result;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A source's input to the consensus calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInput {
    pub source_id: String,
    pub source_name: String,
    pub probability: f64,
    pub accuracy_pct: Option<f64>,
    pub resolved_count: i32,
    pub volume: Option<f64>,
}

/// The result of a consensus calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// The weighted consensus probability.
    pub probability: f64,
    /// Confidence score (0-1) based on source count, agreement, accuracy.
    pub confidence: f64,
    /// Agreement score (0-1), how much sources agree with each other.
    pub agreement: f64,
    /// Number of sources used.
    pub source_count: usize,
    /// Weights assigned to each source.
    pub weights: Vec<SourceWeight>,
    /// Sources that are outliers (>15% from consensus).
    pub outliers: Vec<OutlierSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceWeight {
    pub source_id: String,
    pub source_name: String,
    pub probability: f64,
    pub weight: f64,
    pub accuracy_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierSource {
    pub source_id: String,
    pub source_name: String,
    pub probability: f64,
    pub deviation: f64,
}

/// Minimum resolved questions for a source to get accuracy-based weighting.
const MIN_RESOLVED_FOR_ACCURACY: i32 = 30;

/// Outlier threshold (percentage points from consensus).
const OUTLIER_THRESHOLD: f64 = 0.15;

/// Calculate the Forecise Consensus from multiple source inputs.
pub fn calculate_consensus(sources: &[SourceInput]) -> Result<ConsensusResult> {
    if sources.is_empty() {
        anyhow::bail!("No sources provided for consensus calculation");
    }

    if sources.len() == 1 {
        let s = &sources[0];
        return Ok(ConsensusResult {
            probability: s.probability,
            confidence: 0.3, // Low confidence with single source
            agreement: 1.0,
            source_count: 1,
            weights: vec![SourceWeight {
                source_id: s.source_id.clone(),
                source_name: s.source_name.clone(),
                probability: s.probability,
                weight: 1.0,
                accuracy_pct: s.accuracy_pct,
            }],
            outliers: vec![],
        });
    }

    // Step 1: Calculate weights based on accuracy
    let weights = calculate_weights(sources);

    // Step 2: Compute weighted average
    let consensus_prob: f64 = sources.iter()
        .zip(weights.iter())
        .map(|(s, w)| s.probability * w)
        .sum();

    // Step 3: Calculate agreement (inverse of variance)
    let variance: f64 = sources.iter()
        .zip(weights.iter())
        .map(|(s, w)| w * (s.probability - consensus_prob).powi(2))
        .sum();
    let agreement = (1.0 - variance.sqrt().min(1.0)).max(0.0);

    // Step 4: Detect outliers
    let mut outliers = Vec::new();
    for source in sources {
        let deviation = (source.probability - consensus_prob).abs();
        if deviation > OUTLIER_THRESHOLD {
            outliers.push(OutlierSource {
                source_id: source.source_id.clone(),
                source_name: source.source_name.clone(),
                probability: source.probability,
                deviation,
            });
        }
    }

    // Step 5: Calculate confidence score
    let confidence = calculate_confidence(sources, agreement);

    // Step 6: Build weight details
    let weight_details: Vec<SourceWeight> = sources.iter()
        .zip(weights.iter())
        .map(|(s, w)| SourceWeight {
            source_id: s.source_id.clone(),
            source_name: s.source_name.clone(),
            probability: s.probability,
            weight: *w,
            accuracy_pct: s.accuracy_pct,
        })
        .collect();

    Ok(ConsensusResult {
        probability: consensus_prob.clamp(0.0, 1.0),
        confidence,
        agreement,
        source_count: sources.len(),
        weights: weight_details,
        outliers,
    })
}

/// Calculate normalized weights based on accuracy scores.
/// Sources with more resolved questions and higher accuracy get higher weights.
fn calculate_weights(sources: &[SourceInput]) -> Vec<f64> {
    let raw_weights: Vec<f64> = sources.iter()
        .map(|s| {
            if s.resolved_count >= MIN_RESOLVED_FOR_ACCURACY {
                // Use accuracy as weight (default to 50% if unknown)
                let accuracy = s.accuracy_pct.unwrap_or(50.0) / 100.0;
                // Boost for more resolved questions (logarithmic)
                let volume_boost = (s.resolved_count as f64).ln().max(1.0) / 5.0;
                accuracy * (1.0 + volume_boost)
            } else {
                // Not enough data: use equal weighting with a small base
                0.5
            }
        })
        .collect();

    // Normalize weights to sum to 1
    let sum: f64 = raw_weights.iter().sum();
    if sum == 0.0 {
        vec![1.0 / sources.len() as f64; sources.len()]
    } else {
        raw_weights.iter().map(|w| w / sum).collect()
    }
}

/// Calculate confidence score (0-1) based on:
/// - Number of sources (more = better)
/// - Agreement between sources
/// - Average accuracy of sources
/// - Volume/liquidity
fn calculate_confidence(sources: &[SourceInput], agreement: f64) -> f64 {
    // Source count factor (diminishing returns)
    let count_factor = (sources.len() as f64 / 5.0).min(1.0);

    // Average accuracy factor
    let accuracies: Vec<f64> = sources.iter()
        .filter_map(|s| s.accuracy_pct)
        .collect();
    let accuracy_factor = if accuracies.is_empty() {
        0.5
    } else {
        (accuracies.iter().sum::<f64>() / accuracies.len() as f64 / 100.0).min(1.0)
    };

    // Volume factor (log scale)
    let total_volume: f64 = sources.iter()
        .filter_map(|s| s.volume)
        .sum();
    let volume_factor = if total_volume > 0.0 {
        (total_volume.log10() / 7.0).clamp(0.0, 1.0) // $10M = 1.0
    } else {
        0.3
    };

    // Weighted combination
    let confidence = 0.25 * count_factor
        + 0.30 * agreement
        + 0.25 * accuracy_factor
        + 0.20 * volume_factor;

    confidence.clamp(0.0, 1.0)
}

/// Helper to convert consensus result probabilities to BigDecimal.
pub fn probability_to_decimal(prob: f64) -> BigDecimal {
    BigDecimal::from_str(&format!("{:.6}", prob)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sources() -> Vec<SourceInput> {
        vec![
            SourceInput {
                source_id: "polymarket".into(),
                source_name: "Polymarket".into(),
                probability: 0.67,
                accuracy_pct: Some(89.2),
                resolved_count: 134,
                volume: Some(5_000_000.0),
            },
            SourceInput {
                source_id: "kalshi".into(),
                source_name: "Kalshi".into(),
                probability: 0.61,
                accuracy_pct: Some(81.3),
                resolved_count: 67,
                volume: Some(2_000_000.0),
            },
            SourceInput {
                source_id: "metaculus".into(),
                source_name: "Metaculus".into(),
                probability: 0.72,
                accuracy_pct: Some(84.7),
                resolved_count: 89,
                volume: None,
            },
        ]
    }

    #[test]
    fn test_consensus_calculation() {
        let sources = test_sources();
        let result = calculate_consensus(&sources).unwrap();

        // Consensus should be weighted towards Polymarket (highest accuracy)
        assert!(result.probability > 0.60 && result.probability < 0.75);
        assert!(result.confidence > 0.0 && result.confidence <= 1.0);
        assert_eq!(result.source_count, 3);
        assert!(result.agreement > 0.5); // Sources are reasonably close
    }

    #[test]
    fn test_single_source() {
        let sources = vec![SourceInput {
            source_id: "poly".into(),
            source_name: "Polymarket".into(),
            probability: 0.65,
            accuracy_pct: Some(85.0),
            resolved_count: 100,
            volume: Some(1_000_000.0),
        }];
        let result = calculate_consensus(&sources).unwrap();
        assert!((result.probability - 0.65).abs() < 1e-10);
        assert_eq!(result.confidence, 0.3);
    }

    #[test]
    fn test_outlier_detection() {
        let sources = vec![
            SourceInput {
                source_id: "a".into(),
                source_name: "A".into(),
                probability: 0.70,
                accuracy_pct: Some(90.0),
                resolved_count: 100,
                volume: Some(5_000_000.0),
            },
            SourceInput {
                source_id: "b".into(),
                source_name: "B".into(),
                probability: 0.68,
                accuracy_pct: Some(85.0),
                resolved_count: 80,
                volume: Some(3_000_000.0),
            },
            SourceInput {
                source_id: "c".into(),
                source_name: "C".into(),
                probability: 0.45, // Outlier!
                accuracy_pct: Some(64.0),
                resolved_count: 48,
                volume: Some(500_000.0),
            },
        ];

        let result = calculate_consensus(&sources).unwrap();
        assert!(!result.outliers.is_empty(), "Should detect outlier source C");
    }

    #[test]
    fn test_weights_normalize() {
        let sources = test_sources();
        let weights = calculate_weights(&sources);
        let sum: f64 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "Weights should sum to 1.0");
    }

    #[test]
    fn test_empty_sources() {
        let result = calculate_consensus(&[]);
        assert!(result.is_err());
    }
}
