use std::collections::HashSet;

use syn::Type;

pub(crate) type GenericParamSet = HashSet<String>;

pub(crate) static ALLOWED_PRIMITIVE_TYPES: [&str; 3] = ["u32", "i32", "f32"];

pub(crate) fn is_valid_type(ty: &Type, generic_param_set: &GenericParamSet) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident = segment.ident.to_string();
                // we dont check the generic type here as the kernel function is enforced to have a trait bound of KernelStructMarker for the generic type
                return ALLOWED_PRIMITIVE_TYPES.contains(&ident.as_str())
                    || generic_param_set.contains(&ident);
            }
            false
        }
        Type::Array(arr) => {
            return is_valid_type(&*arr.elem, generic_param_set);
        }
        _ => false,
    }
}

// Helper function to check if the argument type is `u32`
pub(crate) fn is_u32(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();
            return ident == "u32";
        }
    }
    false
}

mod test {
    use syn::parse_quote;
    #[test]
    fn test_is_valid_type() {
        let generic_param_set = std::collections::HashSet::new();
        use super::is_valid_type;
        let valid_type = parse_quote! { u32 };
        let invalid_type = parse_quote! { u64 };
        assert_eq!(is_valid_type(&valid_type, &generic_param_set), true);
        assert_eq!(is_valid_type(&invalid_type, &generic_param_set), false);
    }
}
