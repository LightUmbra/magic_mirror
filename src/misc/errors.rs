use reqwest::StatusCode;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum SettingsError {
    IOError {error_desc: String},
    SerdeError {error_desc: String}
}

#[derive(Debug, Clone)]
pub enum UIError {
    DataError {msg: String},
    APIError {msg: String},
}

impl From<RequestError> for UIError {
    fn from(error: RequestError) -> UIError {
        dbg!(&error);

        UIError::APIError { msg: format!("{:?}", error) }
    }
}

impl From<std::boxed::Box<dyn Error>> for UIError {
    fn from(error: std::boxed::Box<dyn Error>) -> UIError {
        dbg!(&error);

        UIError::APIError { msg: format!("{:?}", error) }
    }
}

#[derive(Debug, Clone)]
pub struct RequestError {
    pub http_code: StatusCode,
    pub details: String,
}

impl RequestError {
    pub fn new(status_code: StatusCode) -> RequestError {
        RequestError {
            http_code: status_code,
            details: get_http_status_description(status_code),
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        &self.details
    }
}

fn get_http_status_description(status_code: StatusCode) -> String {
    let status_description: &str = match status_code {
        StatusCode::CONTINUE => "Continue",
        StatusCode::SWITCHING_PROTOCOLS => "Switching Protocols",
        StatusCode::PROCESSING => "Processing",
        StatusCode::OK => "OK",
        StatusCode::CREATED => "Created",
        StatusCode::ACCEPTED => "Accepted",
        StatusCode::NON_AUTHORITATIVE_INFORMATION => "Non Authoritative Information",
        StatusCode::NO_CONTENT => "No Content",
        StatusCode::RESET_CONTENT => "Reset Content",
        StatusCode::PARTIAL_CONTENT => "Partial Content",
        StatusCode::MULTI_STATUS => "Multi-Status",
        StatusCode::ALREADY_REPORTED => "Already Reported",
        StatusCode::IM_USED => "I'm Used",
        StatusCode::MULTIPLE_CHOICES => "Multiple Choices",
        StatusCode::MOVED_PERMANENTLY => "Moved Permanently",
        StatusCode::FOUND => "Found",
        StatusCode::SEE_OTHER => "See Other",
        StatusCode::NOT_MODIFIED => "Not Modified",
        StatusCode::USE_PROXY => "Use Proxy",
        StatusCode::TEMPORARY_REDIRECT => "Temporary Redirect",
        StatusCode::PERMANENT_REDIRECT => "Permanent Redirect",
        StatusCode::BAD_REQUEST => "Bad Request",
        StatusCode::UNAUTHORIZED => "Unauthorized",
        StatusCode::PAYMENT_REQUIRED => "Payment Required",
        StatusCode::FORBIDDEN => "Forbidden",
        StatusCode::NOT_FOUND => "Not Found",
        StatusCode::METHOD_NOT_ALLOWED => "Method Not Allowed",
        StatusCode::NOT_ACCEPTABLE => "Not Acceptable",
        StatusCode::PROXY_AUTHENTICATION_REQUIRED => "Proxy Authentication Required",
        StatusCode::REQUEST_TIMEOUT => "Request Timeout",
        StatusCode::CONFLICT => "Conflict",
        StatusCode::GONE => "Gone",
        StatusCode::LENGTH_REQUIRED => "Length Required",
        StatusCode::PRECONDITION_FAILED => "Precondition Failed",
        StatusCode::PAYLOAD_TOO_LARGE => "Payload Too Large",
        StatusCode::URI_TOO_LONG => "URI Too Long",
        StatusCode::UNSUPPORTED_MEDIA_TYPE => "Unsupported Media Type",
        StatusCode::RANGE_NOT_SATISFIABLE => "Range Not Satisfiable",
        StatusCode::EXPECTATION_FAILED => "Expectation Failed",
        StatusCode::IM_A_TEAPOT => "I'm a teapot",
        StatusCode::MISDIRECTED_REQUEST => "Misdirected Request",
        StatusCode::UNPROCESSABLE_ENTITY => "Unprocessable Entity",
        StatusCode::LOCKED => "Locked",
        StatusCode::FAILED_DEPENDENCY => "Failed Dependency",
        StatusCode::UPGRADE_REQUIRED => "Upgrade Required",
        StatusCode::PRECONDITION_REQUIRED => "Precondition Required",
        StatusCode::TOO_MANY_REQUESTS => "Too Many Requests",
        StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE => "Request Header Fields Too Large",
        StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => "Unavailable For Legal Reasons",
        StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error",
        StatusCode::NOT_IMPLEMENTED => "Not Implemented",
        StatusCode::BAD_GATEWAY => "Bad Gateway",
        StatusCode::SERVICE_UNAVAILABLE => "Service Unavailable",
        StatusCode::GATEWAY_TIMEOUT => "Gateway Timeout",
        StatusCode::HTTP_VERSION_NOT_SUPPORTED => "HTTP Version Not Supported",
        StatusCode::VARIANT_ALSO_NEGOTIATES => "Variant Also Negotiates",
        StatusCode::INSUFFICIENT_STORAGE => "Insufficient Storage",
        StatusCode::LOOP_DETECTED => "Loop Detected",
        StatusCode::NOT_EXTENDED => "Not Extended",
        StatusCode::NETWORK_AUTHENTICATION_REQUIRED => "Network Authentication Required",
        _ => {
            panic!("Invalid Status Code {}", status_code.as_str())
        }
    };
    status_description.to_string()
}
