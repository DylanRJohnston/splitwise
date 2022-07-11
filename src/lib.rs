use proc_macro::TokenStream;

use quote::{quote_spanned, spanned::Spanned, ToTokens};
use syn::{
    visit_mut::{self, VisitMut},
    Expr, ExprCall, FieldValue,
};

fn compiler_error(error: syn::Error) -> Expr {
    Expr::Verbatim(error.to_compile_error())
}

struct Own {}

impl VisitMut for Own {
    fn visit_field_value_mut(&mut self, node: &mut FieldValue) {
        let expr = &node.expr;
        let span = node.expr.__span();

        match node.expr {
            Expr::Struct(_) => {}
            Expr::Call(_) => {}
            _ => {
                node.expr = syn::parse2(quote_spanned!(span => #expr.to_owned()))
                    .unwrap_or_else(compiler_error)
            }
        }

        visit_mut::visit_field_value_mut(self, node);
    }

    fn visit_expr_call_mut(&mut self, i: &mut ExprCall) {
        for arg in i.args.iter_mut() {
            let span = arg.__span();

            *arg =
                syn::parse2(quote_spanned!(span => #arg.to_owned())).unwrap_or_else(compiler_error)
        }

        visit_mut::visit_expr_call_mut(self, i);
    }
}

#[proc_macro]
pub fn own(tokens: TokenStream) -> TokenStream {
    let mut value: Expr = syn::parse(tokens).unwrap_or_else(compiler_error);

    Own {}.visit_expr_mut(&mut value);

    proc_macro::TokenStream::from(value.into_token_stream())
}

#[cfg(test)]
mod macro_test {
    use quote::quote;
    use syn::visit_mut::VisitMut;
    use syn::{parse2, ExprStruct};

    use crate::Own;

    #[test]
    fn test() {
        let code = quote! {
            namespace::Foo {
                bar: Bar {
                    bar: "Bar",
                    quz: Some("Quz")
                },
                baz: "Baz",
            }
        };

        let expected = quote! {
            namespace::Foo {
                bar: Bar {
                    bar: "Bar".to_owned(),
                    quz: Some("Quz".to_owned())
                },
                baz: "Baz".to_owned(),
            }
        };

        let mut syntax_tree: ExprStruct = parse2(code).unwrap();

        Own {}.visit_expr_struct_mut(&mut syntax_tree);

        assert_eq!(
            format!("{}", quote!(#expected)),
            format!("{}", quote!(#syntax_tree))
        );
    }
}
