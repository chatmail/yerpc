use typescript_type_def::type_expr::TypeInfo;

pub struct Method {
    pub is_notification: bool,
    pub is_positional: bool,
    pub ts_name: String,
    pub rpc_name: String,
    pub args: Vec<(String, &'static TypeInfo)>,
    pub output: Option<&'static TypeInfo>,
    pub docs: Option<String>,
}

impl Method {
    pub fn new(
        ts_name: &str,
        rpc_name: &str,
        args: Vec<(String, &'static TypeInfo)>,
        output: Option<&'static TypeInfo>,
        is_notification: bool,
        is_positional: bool,
        docs: Option<&str>,
    ) -> Self {
        Self {
            ts_name: ts_name.to_string(),
            rpc_name: rpc_name.to_string(),
            args,
            output,
            is_notification,
            is_positional,
            docs: docs.map(|d| d.to_string()),
        }
    }
}
