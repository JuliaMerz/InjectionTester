use async_trait::async_trait;
use core::fmt::Debug;
use dyn_clone::DynClone;
use quick_error::quick_error;
use std::convert::From;

use serde_json;

quick_error! {
    #[derive(Debug)]
    pub enum FilterError {
        DetectedHijack(err: String) {
            display("Detection Reason: {:?}", err)
            from()
        }
    }
}

// Mitigation internal state NEEDS to use ARCs and locks.
// Because they WILL be run in parallel, and must be clonable.
#[async_trait]
pub trait MitigationTechnique: DynClone + Send {
    // Setup function — note this function may be invoked multiple times, and must
    // check whether it's been run before internally.
    async fn run_once(&mut self) -> Result<(), String>;
    async fn filter(&self, prompt: String) -> Result<String, FilterError>;
    fn info(&self) -> String;
}

dyn_clone::clone_trait_object!(MitigationTechnique);

impl Debug for dyn MitigationTechnique {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MitigationTechnique{{{}}}", self.info())
    }
}
