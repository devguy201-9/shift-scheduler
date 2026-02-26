use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StaffGroup {
    pub id: Uuid,
    pub name: String,
    pub parent_group_id: Option<Uuid>,
}
