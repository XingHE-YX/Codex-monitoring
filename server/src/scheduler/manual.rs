use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualRunDecision {
    Accepted { run_id: String },
    Reused { run_id: String },
    RateLimited { retry_after_seconds: u64 },
}

#[derive(Debug, Default)]
struct ManualRunState {
    active_run_id: Option<String>,
    last_accepted_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct ManualRunGate {
    cooldown: Duration,
    state: Mutex<ManualRunState>,
}

impl ManualRunGate {
    pub fn new(cooldown: Duration) -> Self {
        Self {
            cooldown,
            state: Mutex::new(ManualRunState::default()),
        }
    }

    pub fn request(&self, now: DateTime<Utc>, run_id: impl Into<String>) -> ManualRunDecision {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(active_run_id) = &state.active_run_id {
            return ManualRunDecision::Reused {
                run_id: active_run_id.clone(),
            };
        }

        if let Some(last_accepted_at) = state.last_accepted_at {
            let elapsed = now.signed_duration_since(last_accepted_at);
            if elapsed < self.cooldown {
                let remaining = if elapsed < Duration::zero() {
                    self.cooldown
                } else {
                    self.cooldown - elapsed
                };
                let remaining_milliseconds = remaining.num_milliseconds().max(0) as u64;
                return ManualRunDecision::RateLimited {
                    retry_after_seconds: remaining_milliseconds.div_ceil(1_000),
                };
            }
        }

        let run_id = run_id.into();
        state.active_run_id = Some(run_id.clone());
        state.last_accepted_at = Some(now);
        ManualRunDecision::Accepted { run_id }
    }

    pub fn finish(&self, run_id: &str) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.active_run_id.as_deref() == Some(run_id) {
            state.active_run_id = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, Utc};

    use super::{ManualRunDecision, ManualRunGate};

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn scheduler_manual_gate_reuses_active_run_and_rate_limits_new_runs() {
        let gate = ManualRunGate::new(Duration::minutes(5));
        let started_at = at("2026-08-12T00:00:00Z");

        assert_eq!(
            gate.request(started_at, "run-1"),
            ManualRunDecision::Accepted {
                run_id: "run-1".to_string()
            }
        );
        assert_eq!(
            gate.request(started_at + Duration::minutes(1), "run-2"),
            ManualRunDecision::Reused {
                run_id: "run-1".to_string()
            }
        );

        gate.finish("run-1");
        assert_eq!(
            gate.request(started_at + Duration::minutes(2), "run-2"),
            ManualRunDecision::RateLimited {
                retry_after_seconds: 180
            }
        );
        assert_eq!(
            gate.request(started_at + Duration::minutes(5), "run-3"),
            ManualRunDecision::Accepted {
                run_id: "run-3".to_string()
            }
        );
    }
}
