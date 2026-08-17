use std::collections::HashMap;
use std::sync::LazyLock;

pub const PRICING_VERSION: &str = "2026-08-16.1";

#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub prompt_micros_per_token: u64,
    pub completion_micros_per_token: u64,
}

static PRICE_TABLE: LazyLock<HashMap<&'static str, ModelPricing>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "gpt-4o",
        ModelPricing { prompt_micros_per_token: 5, completion_micros_per_token: 15 },
    );
    m.insert(
        "gpt-4o-mini",
        ModelPricing { prompt_micros_per_token: 1, completion_micros_per_token: 2 },
    );
    m.insert(
        "claude-sonnet-5",
        ModelPricing { prompt_micros_per_token: 3, completion_micros_per_token: 15 },
    );
    m.insert(
        "claude-haiku-4.5",
        ModelPricing { prompt_micros_per_token: 1, completion_micros_per_token: 2 },
    );
    m.insert(
        "gemini-2.5-pro",
        ModelPricing { prompt_micros_per_token: 4, completion_micros_per_token: 12 },
    );
    m.insert(
        "gemini-2.5-flash",
        ModelPricing { prompt_micros_per_token: 1, completion_micros_per_token: 3 },
    );
    m
});

pub struct CostEstimator;

impl CostEstimator {
    #[must_use]
    pub fn pricing_for(model: &str) -> ModelPricing {
        PRICE_TABLE
            .get(model)
            .copied()
            .unwrap_or(ModelPricing { prompt_micros_per_token: 5, completion_micros_per_token: 15 })
    }

    #[must_use]
    pub fn estimate_cost_micros(model: &str, prompt_tokens: u32, completion_tokens: u32) -> u64 {
        let p = Self::pricing_for(model);
        u64::from(prompt_tokens) * p.prompt_micros_per_token
            + u64::from(completion_tokens) * p.completion_micros_per_token
    }
}

#[derive(Debug, Clone)]
pub struct CostGuardConfig {
    pub max_cost_per_run_micros: u64,
    pub max_tokens_per_run: u32,
}

impl Default for CostGuardConfig {
    fn default() -> Self {
        Self {
            max_cost_per_run_micros: 50_000_000, // $50
            max_tokens_per_run: 200_000,
        }
    }
}

pub struct CostGuard {
    config: CostGuardConfig,
    prompt_tokens: u32,
    completion_tokens: u32,
    cost_micros: u64,
}

impl CostGuard {
    #[must_use]
    pub fn new(config: CostGuardConfig) -> Self {
        Self { config, prompt_tokens: 0, completion_tokens: 0, cost_micros: 0 }
    }

    pub fn record_usage(&mut self, model: &str, prompt: u32, completion: u32) {
        self.prompt_tokens += prompt;
        self.completion_tokens += completion;
        self.cost_micros += CostEstimator::estimate_cost_micros(model, prompt, completion);
    }

    #[must_use]
    pub fn would_exceed(
        &self,
        model: &str,
        additional_prompt: u32,
        additional_completion: u32,
    ) -> bool {
        let total_tokens =
            self.prompt_tokens + self.completion_tokens + additional_prompt + additional_completion;
        if total_tokens > self.config.max_tokens_per_run {
            return true;
        }
        let projected = self.cost_micros
            + CostEstimator::estimate_cost_micros(model, additional_prompt, additional_completion);
        projected > self.config.max_cost_per_run_micros
    }

    #[must_use]
    pub fn is_exceeded(&self) -> bool {
        let total_tokens = self.prompt_tokens + self.completion_tokens;
        total_tokens > self.config.max_tokens_per_run
            || self.cost_micros > self.config.max_cost_per_run_micros
    }

    #[must_use]
    pub fn total_cost_micros(&self) -> u64 {
        self.cost_micros
    }

    #[must_use]
    pub fn total_tokens(&self) -> u32 {
        self.prompt_tokens + self.completion_tokens
    }
}
