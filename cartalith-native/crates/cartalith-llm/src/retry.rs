//! The one automatic retry (LLM-4; scope s.4 rule 7: "no automatic retry beyond one").
//!
//! WHY: a hosted provider returns transient failures (Anthropic documents 429, 500,
//! 504 and 529 `overloaded_error`) that often succeed a moment later, and a failed
//! first request is the commonest reason a user would otherwise press Send again.
//! Because there is **no enforced spend cap**, an unbounded retry loop is a money
//! risk (scope s.7 risk 2), so the bound is a type, not a convention: a
//! [`RetryBudget`] hands out [`MAX_AUTOMATIC_RETRIES`] permissions and then none.
//!
//! A retry restarts the request from the beginning; it never resumes a stream, and
//! it is allowed only while **nothing has been delivered** to the user: once any
//! text or tool call has been shown, a restart would show it twice (and bill it
//! twice), so the failure is reported instead.
//!
//! MUST NEVER: allow a second retry for one user-initiated request; retry after
//! content was delivered; retry an error that a retry cannot fix (a 400, 401, 403,
//! 404, a malformed chunk, a refused gate).

use crate::error::LlmError;

/// How many times one user-initiated request may be re-sent automatically. The
/// scope's bar, "at most one". Named so a test pins it by a literal.
pub const MAX_AUTOMATIC_RETRIES: u32 = 1;

/// HTTP statuses worth one more try. 429 (rate limit), 500 (provider fault),
/// 502 and 503 (gateway/unavailable, the OpenAI-compatible routers' usual
/// transient answers), 504 (timeout), 529 (Anthropic `overloaded_error`) [429, 500,
/// 504, 529 verified platform.claude.com/docs/en/api/errors; 502 and 503 recalled].
const TRANSIENT_STATUSES: [u16; 6] = [429, 500, 502, 503, 504, 529];

/// True for an error a retry could plausibly fix.
pub fn is_transient(e: &LlmError) -> bool {
    match e {
        LlmError::Provider { status: Some(s), .. } => TRANSIENT_STATUSES.contains(s),
        LlmError::Transport(_) => true,
        _ => false,
    }
}

/// The permission slip for retries of one user-initiated request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetryBudget {
    used: u32,
}

impl RetryBudget {
    /// A fresh budget (a new user-initiated request).
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask to retry after `e`. `delivered_any` is whether any text or tool call
    /// reached the user. True consumes one permission; false changes nothing.
    pub fn try_take(&mut self, e: &LlmError, delivered_any: bool) -> bool {
        if delivered_any || self.used >= MAX_AUTOMATIC_RETRIES || !is_transient(e) {
            return false;
        }
        self.used += 1;
        true
    }

    /// Retries consumed so far.
    pub fn used(&self) -> u32 {
        self.used
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret::Redactor;

    fn provider(status: u16) -> LlmError {
        LlmError::Provider { status: Some(status), message: Redactor::new(None).scrub("x") }
    }

    /// Protects: the bound is exactly one, as a literal. MUTATION: set the const
    /// to 2 and this goes red.
    #[test]
    fn the_bound_is_one() {
        assert_eq!(MAX_AUTOMATIC_RETRIES, 1);
        let mut b = RetryBudget::new();
        assert!(b.try_take(&provider(529), false));
        assert!(!b.try_take(&provider(529), false), "a second retry is never allowed");
        assert!(!b.try_take(&provider(500), false));
        assert_eq!(b.used(), 1);
    }

    /// Protects: which errors are worth a retry, row by row, as literals.
    #[test]
    fn only_transient_errors_qualify() {
        for s in [429u16, 500, 502, 503, 504, 529] {
            assert!(is_transient(&provider(s)), "{s}");
        }
        for s in [400u16, 401, 402, 403, 404, 413, 200] {
            assert!(!is_transient(&provider(s)), "{s}");
        }
        assert!(is_transient(&LlmError::Transport(Redactor::new(None).scrub("reset"))));
        assert!(!is_transient(&LlmError::StreamEndedEarly));
        assert!(!is_transient(&LlmError::MalformedChunk(Redactor::new(None).scrub("x"))));
        assert!(!is_transient(&LlmError::Provider { status: None, message: Redactor::new(None).scrub("x") }));
    }

    /// Protects: once content was delivered no retry is granted, and a refused
    /// attempt does not spend the permission.
    #[test]
    fn nothing_after_delivery_and_refusals_cost_nothing() {
        let mut b = RetryBudget::new();
        assert!(!b.try_take(&provider(529), true));
        assert!(!b.try_take(&provider(401), false));
        assert_eq!(b.used(), 0);
        assert!(b.try_take(&provider(529), false));
    }
}
