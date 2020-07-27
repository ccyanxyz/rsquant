pub trait Strategy {
    fn new(config_path: &str) -> Box<dyn Strategy>
    where
        Self: Sized;
    fn run_forever(&mut self);

    fn name(&self) -> String;
    fn stringify(&self) -> String;
}
