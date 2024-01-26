use model::TokensAmount;
use near_workspaces::result::{ExecutionFailure, ExecutionResult, ExecutionSuccess};

pub(crate) fn calculate_fee(amount: TokensAmount) -> TokensAmount {
    (amount * 5).div_ceil(100)
}

pub(crate) trait PanicFinder {
    fn has_panic(&self, message: &str) -> bool;
}

impl PanicFinder for Result<ExecutionSuccess, ExecutionFailure> {
    fn has_panic(&self, message: &str) -> bool {
        match self {
            Ok(ok) => ok.has_panic(message),
            Err(err) => err.has_panic(message),
        }
    }
}

impl<T> PanicFinder for ExecutionResult<T> {
    fn has_panic(&self, message: &str) -> bool {
        self.outcomes()
            .into_iter()
            .map(|item| match item.clone().into_result() {
                Ok(_) => None,
                Err(err) => Some(err),
            })
            .any(|item| match item {
                None => false,
                Some(err) => format!("{err:?}").contains(message),
            })
    }
}
