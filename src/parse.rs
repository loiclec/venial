use crate::types::*;
use proc_macro2::{Delimiter, Ident, TokenStream, TokenTree};
use std::iter::Peekable;

type TokenIter = Peekable<proc_macro2::token_stream::IntoIter>;

fn parse_ident(token: TokenTree) -> Option<Ident> {
    match token {
        TokenTree::Group(_group) => None,
        TokenTree::Ident(ident) => Some(ident),
        TokenTree::Punct(_punct) => None,
        TokenTree::Literal(_literal) => None,
    }
}

fn consume_attributes(tokens: &mut TokenIter) -> Vec<Attribute> {
    let mut attributes = Vec::new();

    loop {
        match tokens.peek() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '#' => (),
            _ => return attributes,
        };
        let hashbang = tokens.next().unwrap();

        match tokens.peek() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {}
            _ => panic!("cannot parse type"),
        };
        let group = tokens.next().unwrap();

        attributes.push(Attribute {
            tokens: vec![hashbang, group],
        })
    }
}

fn consume_vis_marker(tokens: &mut TokenIter) -> Vec<TokenTree> {
    match tokens.peek() {
        Some(TokenTree::Ident(ident)) if ident == "pub" => {
            let pub_token = tokens.next().unwrap();
            match tokens.peek() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
                    vec![pub_token, tokens.next().unwrap()]
                }
                _ => vec![pub_token],
            }
        }
        Some(TokenTree::Ident(ident)) if ident == "crate" => {
            vec![tokens.next().unwrap()]
        }
        _ => Vec::new(),
    }
}

// TODO - consume_generic_params
// TODO - consume_where_clauses

fn consume_until_period(tokens: &mut TokenIter) -> Vec<TokenTree> {
    let mut tokens_before_period = Vec::new();

    for token in tokens {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == ',' => break,
            _ => {
                tokens_before_period.push(token);
            }
        }
    }

    tokens_before_period
}

pub fn parse_tuple_fields(tokens: TokenStream) -> Vec<TupleField> {
    // TODO - attributes
    // TODO - skip generic params
    let mut fields = Vec::new();

    let mut tokens = tokens.into_iter().peekable();
    loop {
        if tokens.peek().is_none() {
            break;
        }

        consume_attributes(&mut tokens);
        consume_vis_marker(&mut tokens);

        fields.push(TupleField {
            ty: TyExpr {
                tokens: consume_until_period(&mut tokens),
            },
        });
    }

    fields
}

pub fn parse_named_fields(tokens: TokenStream) -> Vec<NamedField> {
    // TODO - attributes
    // TODO - skip generic params
    let mut fields = Vec::new();

    let mut tokens = tokens.into_iter().peekable();
    loop {
        if tokens.peek().is_none() {
            break;
        }

        consume_attributes(&mut tokens);
        consume_vis_marker(&mut tokens);

        let ident = parse_ident(tokens.next().unwrap()).unwrap();

        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ':' => (),
            _ => panic!("cannot parse type"),
        };

        tokens.peek().unwrap();
        fields.push(NamedField {
            name: ident.to_string(),
            ty: TyExpr {
                tokens: consume_until_period(&mut tokens),
            },
        });
    }

    fields
}

pub fn parse_enum_variants(tokens: TokenStream) -> Vec<EnumVariant> {
    // TODO - attributes
    // TODO - skip generic params
    let mut variants = Vec::new();

    let mut tokens = tokens.into_iter().peekable();
    loop {
        if tokens.peek().is_none() {
            break;
        }

        consume_attributes(&mut tokens);

        let ident = parse_ident(tokens.next().unwrap()).unwrap();

        let next_token = tokens.next();
        let contents = match next_token {
            None => StructFields::Unit,
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => StructFields::Unit,
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
                // Consume period, if any
                tokens.next();
                StructFields::Tuple(parse_tuple_fields(group.stream()))
            }
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                // Consume period, if any
                tokens.next();
                StructFields::Named(parse_named_fields(group.stream()))
            }
            _ => panic!("cannot parse type"),
        };

        variants.push(EnumVariant {
            name: ident.to_string(),
            contents,
        });
    }

    variants
}

pub fn parse_type(tokens: TokenStream) -> TypeDeclaration {
    let mut tokens = tokens.into_iter().peekable();

    consume_attributes(&mut tokens);
    consume_vis_marker(&mut tokens);

    if let Some(ident) = parse_ident(tokens.next().unwrap()) {
        if ident == "struct" {
            let struct_name = parse_ident(tokens.next().unwrap()).unwrap();

            let next_token = tokens.next().unwrap();
            let struct_fields = match next_token {
                TokenTree::Punct(punct) if punct.as_char() == ';' => StructFields::Unit,
                TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
                    StructFields::Tuple(parse_tuple_fields(group.stream()))
                }
                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                    StructFields::Named(parse_named_fields(group.stream()))
                }
                _ => panic!("cannot parse type"),
            };

            return TypeDeclaration::Struct(Struct {
                name: struct_name.to_string(),
                fields: struct_fields,
            });
        } else if ident == "enum" {
            let next_token = tokens.next().unwrap();
            let enum_name = parse_ident(next_token).unwrap();

            let next_token = tokens.next().unwrap();
            let enum_variants = match next_token {
                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                    parse_enum_variants(group.stream())
                }
                _ => panic!("cannot parse type"),
            };

            return TypeDeclaration::Enum(Enum {
                name: enum_name.to_string(),
                variants: enum_variants,
            });
        } else {
            panic!("cannot parse type")
        }
    } else {
        panic!("cannot parse type")
    }
}
