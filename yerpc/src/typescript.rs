use std::fmt::Write;
use std::io;
use std::path::Path;
use typescript_type_def::{type_expr::TypeInfo, write_definition_file, DefinitionFileOptions};

pub use typescript_type_def as type_def;
pub use typescript_type_def::TypeDef;

use crate::method::Method;

pub fn typedef_to_expr_string<T: TypeDef>(root_namespace: Option<&str>) -> io::Result<String> {
    let mut expr = vec![];
    <T as TypeDef>::INFO.write_ref_expr(&mut expr, root_namespace)?;
    Ok(String::from_utf8(expr).unwrap())
}

pub fn export_types_to_file<T: TypeDef>(
    path: &Path,
    options: Option<DefinitionFileOptions>,
) -> io::Result<()> {
    let options = options.unwrap_or_else(|| DefinitionFileOptions {
        root_namespace: None,
        ..Default::default()
    });
    let mut file = std::fs::File::create(path)?;
    write_definition_file::<_, T>(&mut file, options)?;
    Ok(())
}

impl Method {
    pub fn to_string_ts(&self, root_namespace: Option<&str>) -> String {
        let (args, call) = if !self.is_positional {
            if let Some((name, ty)) = self.args.first() {
                (
                    format!("{}: {}", name, type_to_expr(ty, root_namespace)),
                    name.to_string(),
                )
            } else {
                ("".to_string(), "undefined".to_string())
            }
        } else {
            let args = self
                .args
                .iter()
                .map(|(name, arg)| format!("{}: {}", name, type_to_expr(arg, root_namespace)))
                .collect::<Vec<String>>()
                .join(", ");
            let call = format!(
                "[{}]",
                self.args
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            (args, call)
        };
        let output = self.output.map_or_else(
            || "void".to_string(),
            |output| type_to_expr(output, root_namespace),
        );
        let (output, inner_method) = if !self.is_notification {
            (format!("Promise<{output}>"), "request")
        } else {
            (output, "notification")
        };
        let docs = if let Some(docs) = &self.docs {
            let docs = docs.split('\n').fold(String::new(), |mut output, s| {
                let _ = writeln!(output, "   *{s}");
                output
            });
            format!("  /**\n{docs}   */")
        } else {
            "".into()
        };
        format!(
            "{}\n  public {}({}): {} {{\n    return (this._transport.{}('{}', {} as RPC.Params)) as {};\n  }}\n\n",
            docs, self.rpc_name_camel, args, output, inner_method, self.rpc_name, call, output
        )
    }
}

fn type_to_expr(ty: &'static TypeInfo, root_namespace: Option<&str>) -> String {
    let mut expr = vec![];
    ty.write_ref_expr(&mut expr, root_namespace).unwrap();
    String::from_utf8(expr).unwrap()
}
