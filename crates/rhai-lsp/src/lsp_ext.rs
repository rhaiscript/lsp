pub mod request {
    use lsp_types::{request::Request, Url};
    use serde::{Deserialize, Serialize};

    pub enum SyntaxTree {}

    #[derive(Serialize, Deserialize)]
    pub struct SyntaxTreeParams {
        pub uri: Url,
    }

    #[derive(Serialize, Deserialize)]
    pub struct SyntaxTreeResult {
        /// Text representation of the syntax tree.
        pub text: String,
        /// JSON representation of the syntax tree.
        pub tree: serde_json::Value,
    }

    impl Request for SyntaxTree {
        type Params = SyntaxTreeParams;

        type Result = Option<SyntaxTreeResult>;

        const METHOD: &'static str = "rhai/syntaxTree";
    }

    pub enum ConvertOffsets {}

    #[derive(Serialize, Deserialize)]
    pub struct ConvertOffsetsParams {
        pub uri: Url,
        pub ranges: Option<Vec<rhai_rowan::TextRange>>,
        pub positions: Option<Vec<rhai_rowan::TextSize>>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ConvertOffsetsResult {
        /// LSP-compatible UTF-16 ranges.
        pub ranges: Option<Vec<lsp_types::Range>>,
        /// LSP-compatible UTF-16 positions.
        pub positions: Option<Vec<lsp_types::Position>>,
    }

    impl Request for ConvertOffsets {
        type Params = ConvertOffsetsParams;

        type Result = Option<ConvertOffsetsResult>;

        const METHOD: &'static str = "rhai/convertOffsets";
    }
}
