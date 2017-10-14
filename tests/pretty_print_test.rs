extern crate webidl;
extern crate zip;

use std::f64;
use std::fs;
use std::io::Read;

use webidl::*;
use webidl::ast::*;
use webidl::visitor::*;

// Test to make sure that Infinity/-Infinity/NaN are correctly pretty printed since they do not
// appear in the Servo WebIDLs.
#[test]
fn pretty_print_float_literals() {
    let ast = vec![
        Definition::Interface(Interface::NonPartial(NonPartialInterface {
            extended_attributes: vec![],
            inherits: None,
            members: vec![
                InterfaceMember::Const(Const {
                    extended_attributes: vec![],
                    name: "const_1".to_string(),
                    nullable: false,
                    type_: ConstType::UnrestrictedDouble,
                    value: ConstValue::FloatLiteral(f64::INFINITY),
                }),
                InterfaceMember::Const(Const {
                    extended_attributes: vec![],
                    name: "const_2".to_string(),
                    nullable: false,
                    type_: ConstType::UnrestrictedDouble,
                    value: ConstValue::FloatLiteral(f64::NEG_INFINITY),
                }),
                InterfaceMember::Const(Const {
                    extended_attributes: vec![],
                    name: "const_3".to_string(),
                    nullable: false,
                    type_: ConstType::UnrestrictedDouble,
                    value: ConstValue::FloatLiteral(f64::NAN),
                }),
            ],
            name: "Test".to_string(),
        })),
    ];
    let mut visitor = PrettyPrintVisitor::new();
    visitor.visit(&ast);
    assert_eq!(
        visitor.get_output(),
        "interface Test {
    const unrestricted double const_1 = Infinity;
    const unrestricted double const_2 = -Infinity;
    const unrestricted double const_3 = NaN;
};\n\n"
    );
}

#[test]
fn pretty_print_servo_webidls() {
    let parser = Parser::new();
    let file = fs::File::open("tests/mozilla_webidls.zip").unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut webidl = String::new();
        file.read_to_string(&mut webidl).unwrap();

        // With the new update to the specification, the "implements" definition has been replaced
        // with "includes", but the Mozilla WebIDLs have not been updated.

        let original_ast = match parser.parse_string(&*webidl) {
            Ok(ast) => ast,
            Err(err) => match err {
                ParseError::UnrecognizedToken {
                    token: Some((_, ref token, _)),
                    ..
                } if *token == Token::Identifier("implements".to_string()) =>
                {
                    continue;
                }
                _ => panic!("parse error: {:?}", err),
            },
        };

        let mut visitor = PrettyPrintVisitor::new();
        visitor.visit(&original_ast);

        // With the new update to the specification, the "implements" definition has been replaced
        // with "includes", but the Mozilla WebIDLs have not been updated. There is some code
        // duplication, but I do not believe it is a big deal since hopefully this is a temporary
        // fix.

        // Compare original AST with AST obtained from pretty print visitor.

        let pretty_print_ast = match parser.parse_string(&*visitor.get_output()) {
            Ok(ast) => ast,
            Err(err) => match err {
                ParseError::UnrecognizedToken {
                    token: Some((_, ref token, _)),
                    ..
                } if *token == Token::Identifier("implements".to_string()) =>
                {
                    continue;
                }
                _ => panic!("parse error: {:?}", err),
            },
        };

        assert_eq!(pretty_print_ast, original_ast);
    }
}
