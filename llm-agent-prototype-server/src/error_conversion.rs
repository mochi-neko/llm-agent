use tonic::{Code, Status};

pub(crate) fn map_anyhow_error_to_grpc_status(error: anyhow::Error) -> Status {
    if let Some(hyper_error) = error.downcast_ref::<hyper::Error>() {
        if hyper_error.is_parse() {
            return Status::new(Code::Internal, "parse error");
        } else if hyper_error.is_parse_too_large() {
            return Status::new(Code::Internal, "parse too large");
        } else if hyper_error.is_parse_status() {
            return Status::new(Code::Internal, "parse status");
        } else if hyper_error.is_user() {
            return Status::new(Code::Internal, "user error");
        } else if hyper_error.is_canceled() {
            return Status::new(Code::Cancelled, "canceled");
        } else if hyper_error.is_closed() {
            return Status::new(Code::Unavailable, "connection closed");
        } else if hyper_error.is_connect() {
            return Status::new(Code::Unavailable, "connection error");
        } else if hyper_error.is_incomplete_message() {
            return Status::new(Code::Internal, "incomplete message");
        } else if hyper_error.is_body_write_aborted() {
            return Status::new(Code::Aborted, "body write aborted");
        } else if hyper_error.is_timeout() {
            return Status::new(Code::DeadlineExceeded, "timeout");
        }
    }

    // If the error is not hyper::Error, use the error message directly.
    Status::new(Code::Internal, format!("Internal error: {:?}", error))
}
