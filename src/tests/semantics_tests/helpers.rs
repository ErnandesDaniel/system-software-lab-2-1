use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;

pub fn analyze(source: &str) -> crate::Result<()> {
    let mut parser = Parser::new(source);
    let program = parser.parse().unwrap();
    let mut analyzer = SemanticsAnalyzer::new();
    analyzer.analyze(&program)
}
