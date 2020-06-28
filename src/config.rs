pub struct EditorConfig {
    pub tab_width: u8,
    pub indentation: IndentationPreference
}

pub enum IndentationPreference {
    Tabs,
    Spaces
}