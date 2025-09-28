use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct ProjectCursor {
    pub last_updated: i64,
    pub id: Uuid,
}

impl ProjectCursor {
    pub fn encode(self) -> String {
        format!("{}:{}", self.last_updated, self.id)
    }
    pub fn decode(s: &str) -> Option<Self> {
        let (ts, id) = s.split_once(":")?;
        let last_updated = ts.parse::<i64>().ok()?;
        let id = Uuid::parse_str(id).ok()?;
        Some(Self { last_updated, id })
    }
}
