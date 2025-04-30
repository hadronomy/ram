//! AST to ItemTree lowering logic.
//!
//! This module handles the conversion from the AST (`ram_syntax::ast`)
//! to the ItemTree (`crate::item_tree::ItemTree`). It extracts definitions
//! of top-level items like modules and labels, ignoring details within
//! function bodies or complex expressions.

use base_db::input::FileId;
use ram_syntax::{AstNode, ast};
use tracing::warn; // Use warn for potentially unattached doc comments

use crate::item_tree::{DocComment, ItemSource, ItemTree, ItemTreeId, LabelDef, ModuleDef};

/// Lowers an AST `Program` node into an `ItemTree`.
///
/// This function serves as the main entry point for converting the raw
/// syntax tree representation of a program into a structured `ItemTree`,
/// which summarizes the top-level definitions.
pub(crate) fn lower_program(program: &ast::Program, file_id: FileId) -> ItemTree {
    let mut lowerer = ItemTreeLowerer::new(file_id);
    lowerer.lower_program_items(program);
    lowerer.finish()
}

/// Helper struct for collecting items from the AST and building the `ItemTree`.
struct ItemTreeLowerer {
    /// Target `ItemTree` being built.
    tree: ItemTree,
    /// File ID being processed.
    file_id: FileId,
    /// Next available ID for items in the tree.
    next_id: u32,
    /// Stores doc comments encountered before their associated item.
    pending_doc_comments: Vec<String>,
}

impl ItemTreeLowerer {
    /// Creates a new `ItemTreeLowerer` for the given file.
    fn new(file_id: FileId) -> Self {
        Self { tree: ItemTree::new(), file_id, next_id: 0, pending_doc_comments: Vec::new() }
    }

    /// Generates a new unique ID for an item within the `ItemTree`.
    fn next_item_id(&mut self) -> ItemTreeId {
        let id = ItemTreeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Processes the statements within a `Program` node to populate the `ItemTree`.
    fn lower_program_items(&mut self, program: &ast::Program) {
        for stmt in program.statements() {
            // Process doc comments first, accumulating them.
            if let Some(doc_comment) = stmt.doc_comment() {
                self.collect_pending_doc_comment(&doc_comment);
                continue; // Move to the next statement after collecting the comment.
            }

            // Process module declarations.
            if let Some(mod_stmt) = stmt.mod_stmt() {
                self.lower_module(&mod_stmt);
            }
            // Process label definitions.
            else if let Some(label_def) = stmt.label_def() {
                self.lower_label(&label_def);
            }
            // If it's not a doc comment or a known item, clear pending comments.
            else {
                self.clear_pending_doc_comments("statement that is not a module or label");
            }
        }
        // Warn about any remaining doc comments at the end of the file.
        self.clear_pending_doc_comments("end of file");
    }

    /// Lowers a module declaration (`ModStmt`) and adds it to the `ItemTree`.
    /// Attaches any pending documentation comments.
    fn lower_module(&mut self, module: &ast::ModStmt) {
        let Some(name) = module.name() else {
            // Module without a name? Log or handle as appropriate.
            warn!(
                "Encountered module statement without a name: {:?}",
                module.syntax().text_range()
            );
            self.clear_pending_doc_comments("unnamed module");
            return;
        };

        let id = self.next_item_id();
        let source = ItemSource { file_id: self.file_id, syntax_node: module.syntax().clone() };

        // name is already a String
        self.tree.modules.push(ModuleDef { name, id, source });
        self.attach_pending_doc_comments(id);
    }

    /// Lowers a label definition (`LabelDef`) and adds it to the `ItemTree`.
    /// Attaches any pending documentation comments.
    fn lower_label(&mut self, label: &ast::LabelDef) {
        let Some(name) = label.name() else {
            // Label without a name? Log or handle as appropriate.
            warn!("Encountered label definition without a name: {:?}", label.syntax().text_range());
            self.clear_pending_doc_comments("unnamed label");
            return;
        };

        let id = self.next_item_id();
        let source = ItemSource { file_id: self.file_id, syntax_node: label.syntax().clone() };

        // name is already a String
        self.tree.labels.push(LabelDef { name, id, source });
        self.attach_pending_doc_comments(id);
    }

    /// Collects the text of a documentation comment, storing it temporarily.
    fn collect_pending_doc_comment(&mut self, doc_comment: &ast::DocComment) {
        if let Some(text) = doc_comment.text() {
            self.pending_doc_comments.push(text);
        }
    }

    /// Attaches any stored pending doc comments to the item with the given `ItemTreeId`.
    fn attach_pending_doc_comments(&mut self, item_id: ItemTreeId) {
        if self.pending_doc_comments.is_empty() {
            return; // Early return if nothing to attach
        }
        for text in self.pending_doc_comments.drain(..) {
            self.tree.doc_comments.push(DocComment { text, item_id });
        }
    }

    /// Clears any pending doc comments, optionally logging a warning.
    fn clear_pending_doc_comments(&mut self, context: &str) {
        if self.pending_doc_comments.is_empty() {
            return; // Early return if nothing to clear
        }
        warn!(
            "Doc comments {:?} were not attached to any item, found before {}",
            self.pending_doc_comments, context
        );
        self.pending_doc_comments.clear();
    }

    /// Finalizes the lowering process and returns the completed `ItemTree`.
    fn finish(self) -> ItemTree {
        // Potential future step: Sort or optimize the tree vectors if needed.
        self.tree
    }
}

// Add SyntaxNodePtr usage to ItemSource in item_tree.rs if not already present
// (Assuming ItemSource definition needs update)
/*
// In crates/hir_def/src/item_tree.rs:
use base_db::input::FileId;
use ram_syntax::SyntaxNodePtr; // Add this import

// ... other imports ...

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemSource {
    pub file_id: FileId,
    // Store a pointer instead of offsets for better resilience to edits
    pub syntax_ptr: SyntaxNodePtr,
}

// ... rest of item_tree.rs ...
*/
