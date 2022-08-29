use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ApiErrorResponse<T>
where
    T: Serialize,
{
    pub message: String,
    pub error: Option<T>,
}

impl<T> From<&'static str> for ApiErrorResponse<T>
where
    T: Serialize,
{
    fn from(v: &'static str) -> Self {
        Self {
            message: v.into(),
            error: None,
        }
    }
}
