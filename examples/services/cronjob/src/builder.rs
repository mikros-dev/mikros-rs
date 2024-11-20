use crate::{Cronjob, CronjobService};

pub struct CronjobBuilder {
    handler: Box<dyn CronjobService>
}

impl CronjobBuilder {
    pub fn new(svc: Box<dyn CronjobService>) -> Self {
        Self {
            handler: svc
        }
    }

    /// Builds the service to be executed.
    pub fn build(self) -> Cronjob {
        Cronjob::new(self.handler)
    }
}