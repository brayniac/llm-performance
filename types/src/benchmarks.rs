// llm-benchmark-types/src/benchmarks.rs
// Benchmark-specific score types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{ValidationError, ValidationResult};

/// Base trait for all benchmark scores
pub trait BenchmarkScore {
    fn benchmark_name(&self) -> &str;
    fn overall_score(&self) -> f64;
    fn timestamp(&self) -> DateTime<Utc>;
    fn validate(&self) -> ValidationResult<()>;
}

/// MMLU-Pro benchmark with detailed subcategories
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MMLUScore {
    pub categories: Vec<MMLUCategoryScore>,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MMLUCategoryScore {
    pub category: String,
    pub score: f64,
    pub total_questions: i32,
    pub correct_answers: i32,
}

/// GSM8K mathematical reasoning benchmark
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GSM8KScore {
    pub problems_solved: i32,
    pub total_problems: i32,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}

/// HumanEval code generation benchmark
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HumanEvalScore {
    pub pass_at_1: f64,
    pub pass_at_10: Option<f64>,
    pub pass_at_100: Option<f64>,
    pub total_problems: i32,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}

/// HellaSwag commonsense reasoning benchmark
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HellaSwagScore {
    pub accuracy: f64,
    pub total_questions: i32,
    pub correct_answers: i32,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}

/// TruthfulQA truthfulness benchmark
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TruthfulQAScore {
    pub truthful_score: f64,
    pub helpful_score: Option<f64>,
    pub total_questions: i32,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}

/// Generic benchmark score for unknown or simple benchmarks
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericBenchmarkScore {
    pub benchmark_name: String,
    pub score: f64,
    pub total_questions: Option<i32>,
    pub correct_answers: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}

/// Enum containing all possible benchmark score types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum BenchmarkScoreType {
    MMLU(MMLUScore),
    GSM8K(GSM8KScore),
    HumanEval(HumanEvalScore),
    HellaSwag(HellaSwagScore),
    TruthfulQA(TruthfulQAScore),
    Generic(GenericBenchmarkScore),
}

// Implement BenchmarkScore trait for all types

impl BenchmarkScore for MMLUScore {
    fn benchmark_name(&self) -> &str {
        "mmlu"
    }

    fn overall_score(&self) -> f64 {
        if self.categories.is_empty() {
            0.0
        } else {
            self.categories.iter().map(|c| c.score).sum::<f64>() / self.categories.len() as f64
        }
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn validate(&self) -> ValidationResult<()> {
        if self.categories.is_empty() {
            return Err(ValidationError::InvalidField {
                field: "categories".to_string(),
                message: "MMLU must have at least one category".to_string(),
            });
        }

        for (i, category) in self.categories.iter().enumerate() {
            if category.category.trim().is_empty() {
                return Err(ValidationError::MissingField {
                    field: format!("categories[{}].category", i),
                });
            }

            if !(0.0..=100.0).contains(&category.score) {
                return Err(ValidationError::OutOfRange {
                    field: format!("categories[{}].score", i),
                    value: category.score.to_string(),
                    range: "0-100".to_string(),
                });
            }

            if category.total_questions <= 0 {
                return Err(ValidationError::OutOfRange {
                    field: format!("categories[{}].total_questions", i),
                    value: category.total_questions.to_string(),
                    range: "> 0".to_string(),
                });
            }

            if category.correct_answers < 0 || category.correct_answers > category.total_questions {
                return Err(ValidationError::OutOfRange {
                    field: format!("categories[{}].correct_answers", i),
                    value: category.correct_answers.to_string(),
                    range: format!("0-{}", category.total_questions),
                });
            }
        }

        Ok(())
    }
}

impl BenchmarkScore for GSM8KScore {
    fn benchmark_name(&self) -> &str {
        "gsm8k"
    }

    fn overall_score(&self) -> f64 {
        if self.total_problems == 0 {
            0.0
        } else {
            (self.problems_solved as f64 / self.total_problems as f64) * 100.0
        }
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn validate(&self) -> ValidationResult<()> {
        if self.total_problems <= 0 {
            return Err(ValidationError::OutOfRange {
                field: "total_problems".to_string(),
                value: self.total_problems.to_string(),
                range: "> 0".to_string(),
            });
        }

        if self.problems_solved < 0 || self.problems_solved > self.total_problems {
            return Err(ValidationError::OutOfRange {
                field: "problems_solved".to_string(),
                value: self.problems_solved.to_string(),
                range: format!("0-{}", self.total_problems),
            });
        }

        Ok(())
    }
}

impl BenchmarkScore for HumanEvalScore {
    fn benchmark_name(&self) -> &str {
        "humaneval"
    }

    fn overall_score(&self) -> f64 {
        self.pass_at_1
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn validate(&self) -> ValidationResult<()> {
        if !(0.0..=100.0).contains(&self.pass_at_1) {
            return Err(ValidationError::OutOfRange {
                field: "pass_at_1".to_string(),
                value: self.pass_at_1.to_string(),
                range: "0-100".to_string(),
            });
        }

        if let Some(pass_at_10) = self.pass_at_10 {
            if !(0.0..=100.0).contains(&pass_at_10) {
                return Err(ValidationError::OutOfRange {
                    field: "pass_at_10".to_string(),
                    value: pass_at_10.to_string(),
                    range: "0-100".to_string(),
                });
            }
        }

        if let Some(pass_at_100) = self.pass_at_100 {
            if !(0.0..=100.0).contains(&pass_at_100) {
                return Err(ValidationError::OutOfRange {
                    field: "pass_at_100".to_string(),
                    value: pass_at_100.to_string(),
                    range: "0-100".to_string(),
                });
            }
        }

        if self.total_problems <= 0 {
            return Err(ValidationError::OutOfRange {
                field: "total_problems".to_string(),
                value: self.total_problems.to_string(),
                range: "> 0".to_string(),
            });
        }

        Ok(())
    }
}

impl BenchmarkScore for HellaSwagScore {
    fn benchmark_name(&self) -> &str {
        "hellaswag"
    }

    fn overall_score(&self) -> f64 {
        self.accuracy
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn validate(&self) -> ValidationResult<()> {
        if !(0.0..=100.0).contains(&self.accuracy) {
            return Err(ValidationError::OutOfRange {
                field: "accuracy".to_string(),
                value: self.accuracy.to_string(),
                range: "0-100".to_string(),
            });
        }

        if self.total_questions <= 0 {
            return Err(ValidationError::OutOfRange {
                field: "total_questions".to_string(),
                value: self.total_questions.to_string(),
                range: "> 0".to_string(),
            });
        }

        if self.correct_answers < 0 || self.correct_answers > self.total_questions {
            return Err(ValidationError::OutOfRange {
                field: "correct_answers".to_string(),
                value: self.correct_answers.to_string(),
                range: format!("0-{}", self.total_questions),
            });
        }

        Ok(())
    }
}

impl BenchmarkScore for TruthfulQAScore {
    fn benchmark_name(&self) -> &str {
        "truthfulqa"
    }

    fn overall_score(&self) -> f64 {
        self.truthful_score
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn validate(&self) -> ValidationResult<()> {
        if !(0.0..=100.0).contains(&self.truthful_score) {
            return Err(ValidationError::OutOfRange {
                field: "truthful_score".to_string(),
                value: self.truthful_score.to_string(),
                range: "0-100".to_string(),
            });
        }

        if let Some(helpful_score) = self.helpful_score {
            if !(0.0..=100.0).contains(&helpful_score) {
                return Err(ValidationError::OutOfRange {
                    field: "helpful_score".to_string(),
                    value: helpful_score.to_string(),
                    range: "0-100".to_string(),
                });
            }
        }

        if self.total_questions <= 0 {
            return Err(ValidationError::OutOfRange {
                field: "total_questions".to_string(),
                value: self.total_questions.to_string(),
                range: "> 0".to_string(),
            });
        }

        Ok(())
    }
}

impl BenchmarkScore for GenericBenchmarkScore {
    fn benchmark_name(&self) -> &str {
        &self.benchmark_name
    }

    fn overall_score(&self) -> f64 {
        self.score
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn validate(&self) -> ValidationResult<()> {
        if self.benchmark_name.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "benchmark_name".to_string(),
            });
        }

        if !(0.0..=100.0).contains(&self.score) {
            return Err(ValidationError::OutOfRange {
                field: "score".to_string(),
                value: self.score.to_string(),
                range: "0-100".to_string(),
            });
        }

        if let Some(total) = self.total_questions {
            if total <= 0 {
                return Err(ValidationError::OutOfRange {
                    field: "total_questions".to_string(),
                    value: total.to_string(),
                    range: "> 0".to_string(),
                });
            }
        }

        if let Some(correct) = self.correct_answers {
            if correct < 0 {
                return Err(ValidationError::OutOfRange {
                    field: "correct_answers".to_string(),
                    value: correct.to_string(),
                    range: ">= 0".to_string(),
                });
            }

            if let Some(total) = self.total_questions {
                if correct > total {
                    return Err(ValidationError::OutOfRange {
                        field: "correct_answers".to_string(),
                        value: correct.to_string(),
                        range: format!("0-{}", total),
                    });
                }
            }
        }

        Ok(())
    }
}

impl BenchmarkScore for BenchmarkScoreType {
    fn benchmark_name(&self) -> &str {
        match self {
            BenchmarkScoreType::MMLU(score) => score.benchmark_name(),
            BenchmarkScoreType::GSM8K(score) => score.benchmark_name(),
            BenchmarkScoreType::HumanEval(score) => score.benchmark_name(),
            BenchmarkScoreType::HellaSwag(score) => score.benchmark_name(),
            BenchmarkScoreType::TruthfulQA(score) => score.benchmark_name(),
            BenchmarkScoreType::Generic(score) => score.benchmark_name(),
        }
    }

    fn overall_score(&self) -> f64 {
        match self {
            BenchmarkScoreType::MMLU(score) => score.overall_score(),
            BenchmarkScoreType::GSM8K(score) => score.overall_score(),
            BenchmarkScoreType::HumanEval(score) => score.overall_score(),
            BenchmarkScoreType::HellaSwag(score) => score.overall_score(),
            BenchmarkScoreType::TruthfulQA(score) => score.overall_score(),
            BenchmarkScoreType::Generic(score) => score.overall_score(),
        }
    }

    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            BenchmarkScoreType::MMLU(score) => score.timestamp(),
            BenchmarkScoreType::GSM8K(score) => score.timestamp(),
            BenchmarkScoreType::HumanEval(score) => score.timestamp(),
            BenchmarkScoreType::HellaSwag(score) => score.timestamp(),
            BenchmarkScoreType::TruthfulQA(score) => score.timestamp(),
            BenchmarkScoreType::Generic(score) => score.timestamp(),
        }
    }

    fn validate(&self) -> ValidationResult<()> {
        match self {
            BenchmarkScoreType::MMLU(score) => score.validate(),
            BenchmarkScoreType::GSM8K(score) => score.validate(),
            BenchmarkScoreType::HumanEval(score) => score.validate(),
            BenchmarkScoreType::HellaSwag(score) => score.validate(),
            BenchmarkScoreType::TruthfulQA(score) => score.validate(),
            BenchmarkScoreType::Generic(score) => score.validate(),
        }
    }
}

// Helper constructors
impl MMLUScore {
    pub fn new(categories: Vec<MMLUCategoryScore>) -> Self {
        Self {
            categories,
            timestamp: Utc::now(),
            context: None,
        }
    }
}

impl GSM8KScore {
    pub fn new(problems_solved: i32, total_problems: i32) -> Self {
        Self {
            problems_solved,
            total_problems,
            timestamp: Utc::now(),
            context: None,
        }
    }
}

impl HumanEvalScore {
    pub fn new(pass_at_1: f64, total_problems: i32) -> Self {
        Self {
            pass_at_1,
            pass_at_10: None,
            pass_at_100: None,
            total_problems,
            timestamp: Utc::now(),
            context: None,
        }
    }
}

impl HellaSwagScore {
    pub fn new(correct_answers: i32, total_questions: i32) -> Self {
        let accuracy = (correct_answers as f64 / total_questions as f64) * 100.0;
        Self {
            accuracy,
            total_questions,
            correct_answers,
            timestamp: Utc::now(),
            context: None,
        }
    }
}

impl TruthfulQAScore {
    pub fn new(truthful_score: f64, total_questions: i32) -> Self {
        Self {
            truthful_score,
            helpful_score: None,
            total_questions,
            timestamp: Utc::now(),
            context: None,
        }
    }
}

impl GenericBenchmarkScore {
    pub fn new(benchmark_name: String, score: f64) -> Self {
        Self {
            benchmark_name,
            score,
            total_questions: None,
            correct_answers: None,
            timestamp: Utc::now(),
            context: None,
        }
    }
}