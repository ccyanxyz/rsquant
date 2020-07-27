use crate::traits::Strategy;
use log::warn;

#[derive(Debug)]
pub struct Dummy {}

impl Strategy for Dummy {
    fn new(_: &str) -> Box<dyn Strategy> {
        Box::new(Dummy {})
    }

    fn run_forever(&mut self) {
        warn!("dummy!");
    }

    fn name(&self) -> String {
        "dummy".into()
    }

    fn stringify(&self) -> String {
        format!("{:?}", self)
    }
}
