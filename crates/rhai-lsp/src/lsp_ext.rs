pub mod request {
    use lsp_types::{request::Request, Url};
    use serde::{Deserialize, Serialize};

    pub enum HirDump {}

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HirDumpParams {
        pub workspace_uri: Option<Url>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HirDumpResult {
        /// The unstable textual representation
        /// of the HIR.
        pub hir: String,
    }

    impl Request for HirDump {
        type Params = HirDumpParams;

        type Result = Option<HirDumpResult>;

        const METHOD: &'static str = "rhai/hirDump";
    }

    pub enum SyntaxTree {}

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
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
