//! AST lowering logic
//!
//! This module handles the conversion from the AST to the ItemTree.
//! It extracts the necessary information from AST nodes while ignoring
//! details within function bodies.

use base_db::input::FileId;
use ram_syntax::{AstNode, ast};

use crate::item_tree::{DocComment, ItemSource, ItemTree, ItemTreeId, LabelDef, ModuleDef};

/// Lower a Program AST node into an ItemTree
pub(crate) fn lower_program(program: &ast::Program, file_id: FileId) -> ItemTree {
    let mut collector = ItemCollector::new(file_id);
    collector.collect_program(program);
    collector.finish()
}

/// Helper struct for collecting items from the AST
struct ItemCollector {
    /// Target ItemTree being built
    tree: ItemTree,

    /// File ID being processed
    file_id: FileId,

    /// Next available ID for items
    next_id: u32,
}

impl ItemCollector {
    /// Create a new item collector
    fn new(file_id: FileId) -> Self {
        Self { tree: ItemTree::new(), file_id, next_id: 0 }
    }

    /// Generate a new unique item ID
    fn next_item_id(&mut self) -> ItemTreeId {
        let id = ItemTreeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Collect items from a Program node
    fn collect_program(&mut self, program: &ast::Program) {
        for stmt in program.statements() {
            // Process module declarations
            if let Some(mod_stmt) = stmt.mod_stmt() {
                self.collect_module(&mod_stmt);
            }

            // Process label definitions
            if let Some(label_def) = stmt.label_def() {
                self.collect_label(&label_def);
            }

            // Process doc comments
            if let Some(doc_comment) = stmt.doc_comment() {
                self.collect_doc_comment(&doc_comment);
            }
        }
    }

    /// Collect a module declaration
    fn collect_module(&mut self, module: &ast::ModStmt) {
        if let Some(name) = module.name() {
            let syntax = module.syntax();
            let id = self.next_item_id();
            let source = ItemSource {
                file_id: self.file_id,
                start_offset: syntax.text_range().start().into(),
                end_offset: syntax.text_range().end().into(),
            };

            self.tree.modules.push(ModuleDef { name, id, source });
        }
    }

    /// Collect a label definition
    fn collect_label(&mut self, label: &ast::LabelDef) {
        if let Some(name) = label.name() {
            let syntax = label.syntax();
            let id = self.next_item_id();
            let source = ItemSource {
                file_id: self.file_id,
                start_offset: syntax.text_range().start().into(),
                end_offset: syntax.text_range().end().into(),
            };

            self.tree.labels.push(LabelDef { name, id, source });
        }
    }

    /// Collect a documentation comment
    fn collect_doc_comment(&mut self, doc_comment: &ast::DocComment) {
        if let Some(text) = doc_comment.text() {
            // Attach the doc comment to the last item (if any)
            if let Some(item_id) = self
                .tree
                .labels
                .last()
                .map(|l| l.id)
                .or_else(|| self.tree.modules.last().map(|m| m.id))
            {
                self.tree.doc_comments.push(DocComment { text, item_id });
            }
        }
    }

    /// Finish collecting and return the ItemTree
    fn finish(self) -> ItemTree {
        self.tree
    }
}
