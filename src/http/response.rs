use crate::models::Response;

pub fn format_response(response: &Response) -> String {
    response.format()
}
