pub struct Inverters {
    pub time: u64,
    pub inverter_0: Option<f64>,
    pub inverter_1: Option<f64>,
}

pub struct Combined {
    pub time: u64,
    pub combined: Option<f64>,
}
