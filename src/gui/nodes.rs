#[derive(Clone)]
pub struct NodeInfo {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f32>,
    pub last_seen: chrono::DateTime<chrono::Local>,
}