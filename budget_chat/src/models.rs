#[derive(Clone, Debug)]
pub struct Message {
    pub sender_id: usize,
    pub contents: String,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Member {
    pub id: usize,
    pub name: String,
}
