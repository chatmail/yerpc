use syn::{GenericArgument, PathArguments, Type};

pub fn is_result_ty(ty: &Type) -> bool {
    if let Type::Path(path) = ty
        && let Some(last) = path.path.segments.last()
        && last.ident == "Result"
    {
        return true;
    }
    false
}

pub fn extract_result_ty(ty: &Type) -> &Type {
    if let Type::Path(path) = ty
        && let Some(last) = path.path.segments.last()
        && last.ident == "Result"
        && let PathArguments::AngleBracketed(ref generics) = last.arguments
        && let Some(GenericArgument::Type(inner_ty)) = generics.args.first()
    {
        return inner_ty;
    }
    ty
}

// pub fn ty_ident(name: &str) -> Ident {
//     Ident::new(&name.to_case(Case::UpperCamel), Span::call_site())
// }
