use std::io::Read;
use crate::binary_parser::IonBinaryParser;
use crate::ion_parser_types::*;
use crate::binary_parser_types::*;
use crate::symbol_table::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct IonParser<T: Read> {
    parser: IonBinaryParser<T>,
    context: SymbolContext,
}

impl <T: Read>IonParser<T> {
    pub fn new(reader: T) -> IonParser<T> {
        IonParser { 
            parser: IonBinaryParser::new(reader),
            context: SymbolContext::new(),
        }
    }

    pub fn consume_value(&mut self) -> Result<IonValue, IonParserError> {
        let value_header = self.parser.consume_value_header()?;

        match value_header.r#type {
            ValueType::Bool(value) =>  {
                Ok(IonValue::Bool(value))
            },
            ValueType::Annotation => {
                self.consume_annotation(&value_header)
            },
            _ => Err(IonParserError::Unimplemented)
        }
    }

    pub fn consume_annotation(&mut self, header: &ValueHeader) -> Result<IonValue, IonParserError> {
        let length = match header.length {
            ValueLength::LongLength => self.parser.consume_varuint()?.0,
            ValueLength::ShortLength(len) => len.into(),
            ValueLength::NullValue => return Err(IonParserError::NullAnnotationFound),
        };

        let mut remaining_annot_bytes = self.parser.consume_varuint()?.0;

        let mut symbols = Vec::new();

        while remaining_annot_bytes > 0 {
            let (annot, consumed_bytes) = self.parser.consume_varuint()?;

            symbols.push(annot);

            remaining_annot_bytes = match remaining_annot_bytes.checked_sub(consumed_bytes as u64) {
                Some(result) => result,
                None => return Err(IonParserError::BadFormatLengthFound) 
            }
        }

        let value = self.consume_value()?;

        //TODO: Check annotation symbols in order to know what to do with the content. It can be a symtem table, shared table, etc

        unimplemented!()
    }
}

/*

Basically, for QLDB, the first value that the DB sends is:
Annotation: 
    notation: 3 ($ion_symbol_table which means that is a local symbol table)
    Struct
        Symbols (via the id 7)
        List: Which contains the list of new Symbols

... After consuming the annotation header
Annotation Length: Ok(38)
Annotation annot_length: Ok(1)
Annotation annot: Ok(3)
Annotation value header: Ok(ValueHeader { type: Struct, length: LongLength })
Annotation value length: Ok(34)
Annotation value first key: Ok(7)
Annotation value first value header: Ok(ValueHeader { type: List, length: LongLength })

In the list, symbols are added in consecutive IDs following their insert order. 
A symbol cannot replace an already existing symbol. So, the system symbols come first, 
later the imported symbols, and finally, the local symbols. 
 */
