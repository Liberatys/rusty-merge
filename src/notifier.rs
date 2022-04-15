#[derive(Debug)]
pub struct Notification {
    title: String,
    body: String,
    icon: String,
}

impl Notification {
    pub fn new(title: String, body: String, icon: String) -> Self {
        Self { title, body, icon }
    }

    pub fn push(&self) -> Result<(), ()> {
        Ok(())
    }
}
