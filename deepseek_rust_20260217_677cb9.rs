pub trait DigitalTwin {
    fn predict_trajectory(&self, protocol: &BiofieldProtocol, start: &BiophysicalState) -> Vec<BiophysicalState>;
    fn update(&mut self, new_data: &[(Protocol, BiophysicalState, BiophysicalState)]);
}