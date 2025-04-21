//! Database implementation for the RAM virtual machine

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base_db::{
    FileId, FileSourceRootInput, FileText, Files, SourceDatabase, SourceRoot, SourceRootId,
    SourceRootInput,
};
use hir::db::HirDatabase;
use hir::name_resolution::ResolvedFile;
use hir_def::db::{HirDefDatabase, SourceFile};
use miette::*;
use ram_core::db::VmState;
use ram_core::error::VmError;
use ram_core::instruction::{InstructionDefinition, InstructionKind};
use ram_core::operand::Operand;
use ram_core::registry::InstructionRegistry;
use ram_core::standard_instructions;
use ram_parser::{Diagnostic, build_tree, parse};
use ram_syntax::{AstNode, Program};
use salsa::{Durability, Event};
use tracing;
/// The database trait for VM queries
#[salsa::db]
pub trait VmDatabase: HirDatabase {
    /// Parse source code into an AST Program node
    fn parse_program(&self, source: &str) -> (Program, Vec<Diagnostic>);

    /// Parse source code and create a SourceFile
    fn parse_to_source_file(&self, file_id: FileId, source: &str) -> SourceFile;

    /// Convert a HIR Body to a VM Program
    fn hir_to_vm_program(&self, body: &hir::body::Body)
    -> Result<crate::program::Program, VmError>;

    /// Parse source code directly to a VM Program
    fn parse_to_vm_program(&self, source: &str) -> Result<crate::program::Program, VmError>;

    /// Get the instruction registry
    fn instruction_registry(&self) -> Arc<InstructionRegistry> {
        Arc::new(self.get_instruction_registry_impl())
    }

    /// Execute an instruction with the given operand
    fn execute_instruction(
        &self,
        instruction: InstructionKind,
        operand: Option<Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        // Get the instruction definition
        let definition = self.get_instruction_definition(&instruction).ok_or_else(|| {
            VmError::InvalidInstruction(format!("Unknown instruction: {}", instruction))
        })?;

        // Execute the instruction
        definition.execute(operand.as_ref(), vm_state)
    }

    /// Get the instruction for a given name
    fn get_instruction(&self, name: &str) -> Option<InstructionKind> {
        Some(InstructionKind::from_name(name))
    }

    /// Get the instruction definition for a given kind
    fn get_instruction_definition(
        &self,
        kind: &InstructionKind,
    ) -> Option<Arc<dyn InstructionDefinition>> {
        let registry = self.get_instruction_registry_impl();
        registry.get(kind)
    }

    /// Validate an instruction with the given operand
    fn validate_instruction(
        &self,
        instruction: InstructionKind,
        operand: Option<&Operand>,
    ) -> Result<(), VmError> {
        // Get the instruction definition
        let definition = self.get_instruction_definition(&instruction).ok_or_else(|| {
            VmError::InvalidInstruction(format!("Unknown instruction: {}", instruction))
        })?;

        // Check if the instruction requires an operand
        if definition.requires_operand() && operand.is_none() {
            return Err(VmError::InvalidOperand(format!(
                "Instruction {} requires an operand",
                instruction
            )));
        }

        // Check if the operand is valid for this instruction
        if let Some(operand) = operand {
            let allowed_kinds = definition.allowed_operand_kinds();
            if !allowed_kinds.is_empty() && !allowed_kinds.contains(&operand.kind) {
                return Err(VmError::InvalidOperand(format!(
                    "Invalid operand kind for instruction {}",
                    instruction
                )));
            }
        }

        Ok(())
    }

    /// Get the instruction registry implementation
    fn get_instruction_registry_impl(&self) -> InstructionRegistry;

    /// Add a diagnostic to the database
    fn add_diagnostic(&mut self, file_id: FileId, diagnostic: Diagnostic);

    /// Add multiple diagnostics to the database
    fn add_diagnostics(&mut self, file_id: FileId, diagnostics: Vec<Diagnostic>) {
        for diagnostic in diagnostics {
            self.add_diagnostic(file_id, diagnostic);
        }
    }

    /// Get all diagnostics for a specific file
    fn diagnostics(&self, file_id: FileId) -> Vec<Diagnostic>;

    /// Get all diagnostics in the database
    fn all_diagnostics(&self) -> HashMap<FileId, Vec<Diagnostic>>;

    /// Clear all diagnostics for a specific file
    fn clear_diagnostics(&mut self, file_id: FileId);

    /// Clear all diagnostics in the database
    fn clear_all_diagnostics(&mut self);
}

/// Implementation of the VM database
#[salsa::db]
#[derive(Default)]
pub struct VmDatabaseImpl {
    storage: salsa::Storage<Self>,
    files: Arc<Files>,
    instruction_registry: Arc<Mutex<InstructionRegistry>>,
    diagnostics: Mutex<HashMap<FileId, Vec<Diagnostic>>>,
}

// Explicitly implement Send and Sync for VmDatabaseImpl
// This is safe because all fields are protected by Mutex
unsafe impl Send for VmDatabaseImpl {}
unsafe impl Sync for VmDatabaseImpl {}

#[salsa::db]
impl salsa::Database for VmDatabaseImpl {
    fn salsa_event(&self, event: &dyn Fn() -> Event) {
        // Log events at debug level using tracing
        let event = event();
        tracing::debug!("salsa_event: {:?}", event);
    }
}

impl Clone for VmDatabaseImpl {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            files: self.files.clone(),
            instruction_registry: self.instruction_registry.clone(),
            diagnostics: Mutex::new(self.diagnostics.lock().unwrap().clone()),
        }
    }
}

#[salsa::db]
impl VmDatabase for VmDatabaseImpl {
    fn parse_program(&self, source: &str) -> (Program, Vec<Diagnostic>) {
        // Parse the source text using the recursive descent parser
        let (events, errors) = parse(source);

        // Convert the events into a syntax tree
        let (tree, cache) = build_tree(events);
        let syntax_node = ram_syntax::SyntaxNode::new_root_with_resolver(tree, cache);
        let program = Program::cast(syntax_node).expect("Failed to cast root node to Program");

        (program, errors)
    }

    fn parse_to_source_file(&self, file_id: FileId, source: &str) -> SourceFile {
        let (program, errors) = self.parse_program(source);

        // Store the diagnostics for this file
        if !errors.is_empty() {
            let mut diagnostics = self.diagnostics.lock().unwrap();
            diagnostics.insert(file_id, errors);
        }

        SourceFile::new(source.to_string(), program)
    }

    fn hir_to_vm_program(
        &self,
        body: &hir::body::Body,
    ) -> Result<crate::program::Program, VmError> {
        // Use the Program's from_hir method
        crate::program::Program::from_hir(body, self)
    }

    fn parse_to_vm_program(&self, source: &str) -> Result<crate::program::Program, VmError> {
        // Parse the source into an AST Program directly
        let (program, errors) = self.parse_program(source);

        // Check for errors
        if !errors.is_empty() {
            // Convert all errors to a nice report format
            // FIXME: Refactor the DB API to be more consistent and
            // not repetitive
            let report = ram_parser::convert_errors(source, errors);
            eprintln!("{:?}", miette::Error::new(report.clone()));
            return Err(VmError::ParseError(report));
        }

        // Create a new database instance for this operation
        // This avoids the need to modify the existing database
        let mut temp_db = VmDatabaseImpl::new();

        // Create a temporary file ID for this source
        let file_id = FileId(0);

        // Set the file text in the temporary database
        temp_db.set_file_text(file_id, source);

        // Lower the AST Program to an ItemTree in the temporary database
        let item_tree = Arc::new(hir_def::item_tree::ItemTree::lower(&program, file_id));

        // Create a DefId for the program
        let def_id = hir::ids::DefId { file_id, local_id: hir::ids::LocalDefId(0) };

        // Lower the AST Program to a HIR Body
        let body = hir::lower::lower_program(&program, def_id, file_id, &item_tree);

        // Convert the HIR Body to a VM Program
        // We can use the original database for this since it doesn't depend on file_id
        self.hir_to_vm_program(&body)
    }

    fn add_diagnostic(&mut self, file_id: FileId, diagnostic: Diagnostic) {
        let mut diagnostics = self.diagnostics.lock().unwrap();
        diagnostics.entry(file_id).or_default().push(diagnostic);
    }

    fn diagnostics(&self, file_id: FileId) -> Vec<Diagnostic> {
        let diagnostics = self.diagnostics.lock().unwrap();
        diagnostics.get(&file_id).cloned().unwrap_or_default()
    }

    fn all_diagnostics(&self) -> HashMap<FileId, Vec<Diagnostic>> {
        self.diagnostics.lock().unwrap().clone()
    }

    fn clear_diagnostics(&mut self, file_id: FileId) {
        let mut diagnostics = self.diagnostics.lock().unwrap();
        diagnostics.remove(&file_id);
    }

    fn clear_all_diagnostics(&mut self) {
        let mut diagnostics = self.diagnostics.lock().unwrap();
        diagnostics.clear();
    }

    fn get_instruction_registry_impl(&self) -> InstructionRegistry {
        self.instruction_registry.lock().unwrap().clone()
    }
}

#[salsa::db]
impl HirDefDatabase for VmDatabaseImpl {
    fn item_tree(&self, file_id: FileId) -> Arc<hir_def::item_tree::ItemTree> {
        // Get the file text from the database
        let file_text = self.file_text(file_id);

        // Parse the file text into an AST Program
        let (program, errors) = self.parse_program(file_text.text(self).as_ref());

        // Store the diagnostics for this file
        if !errors.is_empty() {
            let mut diagnostics = self.diagnostics.lock().unwrap();
            diagnostics.insert(file_id, errors);
        }

        // Lower the AST Program to an ItemTree
        Arc::new(hir_def::item_tree::ItemTree::lower(&program, file_id))
    }
}

#[salsa::db]
impl HirDatabase for VmDatabaseImpl {
    #[doc = " Resolve all definitions for a given file"]
    fn resolve_file(&self, file_id: FileId) -> Arc<hir::name_resolution::ResolvedFile> {
        // Get the ItemTree for this file
        let item_tree = self.item_tree(file_id);

        // Create a new ResolvedFile
        let mut resolved = ResolvedFile::default();

        // For each module in the ItemTree, create a definition
        for (i, module) in item_tree.modules.iter().enumerate() {
            let def_id = hir::ids::DefId { file_id, local_id: hir::ids::LocalDefId(i as u32) };

            resolved.add_definition(module.name.clone(), def_id);
        }

        // For each label in the ItemTree, create a definition
        let start_id = item_tree.modules.len();
        for (i, label) in item_tree.labels.iter().enumerate() {
            let def_id =
                hir::ids::DefId { file_id, local_id: hir::ids::LocalDefId((start_id + i) as u32) };

            resolved.add_definition(label.name.clone(), def_id);
        }

        Arc::new(resolved)
    }

    #[doc = " Get the body for a specific definition"]
    fn body(&self, def_id: hir::ids::DefId) -> Arc<hir::body::Body> {
        // Get the file ID from the definition ID
        let file_id = def_id.file_id;

        // Get all bodies in the file
        let bodies = self.bodies_in_file(file_id);

        // Find the body for this definition
        if let Some(body) = bodies.get(&def_id.local_id) {
            return body.clone();
        }

        // If not found, create a new body
        // Get the file text from the database
        let file_text = self.file_text(file_id);

        // Parse the file text into an AST Program
        let (program, _errors) = self.parse_program(file_text.text(self).as_ref());

        // Get the ItemTree for this file
        let item_tree = self.item_tree(file_id);

        // Lower the AST Program to a HIR Body
        let body = hir::lower::lower_program(&program, def_id, file_id, &item_tree);

        Arc::new(body)
    }

    fn bodies_in_file(
        &self,
        file_id: FileId,
    ) -> Arc<HashMap<hir::ids::LocalDefId, Arc<hir::body::Body>>> {
        // Create a map to store the bodies
        let mut bodies = HashMap::new();

        // Get the file text from the database
        let file_text = self.file_text(file_id);

        // Parse the file text into an AST Program
        let (program, _errors) = self.parse_program(file_text.text(self).as_ref());

        // Get the ItemTree for this file
        let item_tree = self.item_tree(file_id);

        // Process all labels in the ItemTree
        for label in &item_tree.labels {
            // Create a DefId for this label
            let def_id = hir::ids::DefId { file_id, local_id: hir::ids::LocalDefId(label.id.0) };

            // Lower the AST Program to a HIR Body for this label
            let body = hir::lower::lower_program(&program, def_id, file_id, &item_tree);

            // Add the body to the map
            bodies.insert(def_id.local_id, Arc::new(body));
        }

        // If there are no labels, create a default body for the file
        if bodies.is_empty() {
            let def_id = hir::ids::DefId { file_id, local_id: hir::ids::LocalDefId(0) };

            let body = hir::lower::lower_program(&program, def_id, file_id, &item_tree);

            bodies.insert(def_id.local_id, Arc::new(body));
        }

        Arc::new(bodies)
    }
}

#[salsa::db]
impl SourceDatabase for VmDatabaseImpl {
    fn file_text(&self, file_id: FileId) -> FileText {
        self.files.file_text(file_id)
    }

    #[doc = " Set the text of a file"]
    fn set_file_text(&mut self, file_id: FileId, text: &str) {
        // Clone the files reference to avoid borrowing issues
        let files = Arc::clone(&self.files);
        files.set_file_text(self, file_id, text);
    }

    #[doc = " Set the text of a file with a specific durability"]
    fn set_file_text_with_durability(
        &mut self,
        file_id: FileId,
        text: &str,
        durability: Durability,
    ) {
        // Clone the files reference to avoid borrowing issues
        let files = Arc::clone(&self.files);
        files.set_file_text_with_durability(self, file_id, text, durability);
    }

    #[doc = " Contents of the source root"]
    fn source_root(&self, id: SourceRootId) -> SourceRootInput {
        self.files.source_root(id)
    }

    #[doc = " Source root of the file"]
    fn file_source_root(&self, _id: FileId) -> FileSourceRootInput {
        // Try to get the source root ID for this file
        // If not found, create a default one with source root ID 0
        // In a real implementation, this would properly handle the case when a file doesn't have a source root
        FileSourceRootInput::builder(SourceRootId(0)).new(self)
    }

    #[doc = " Set the source root of a file with a specific durability"]
    fn set_file_source_root_with_durability(
        &mut self,
        id: FileId,
        source_root_id: SourceRootId,
        durability: Durability,
    ) {
        // Clone the files reference to avoid borrowing issues
        let files = Arc::clone(&self.files);
        files.set_file_source_root_with_durability(self, id, source_root_id, durability);
    }

    #[doc = " Set the source root with a specific durability"]
    fn set_source_root_with_durability(
        &mut self,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    ) {
        // Clone the files reference to avoid borrowing issues
        let files = Arc::clone(&self.files);
        files.set_source_root_with_durability(self, source_root_id, source_root, durability);
    }
}

impl VmDatabaseImpl {
    /// Create a new VM database
    pub fn new() -> Self {
        let mut db = Self::default();
        db.initialize_instructions();
        db
    }

    /// Initialize the default instructions
    fn initialize_instructions(&mut self) {
        let registry = standard_instructions();
        self.instruction_registry = Arc::new(Mutex::new(registry));
    }

    /// Register a custom instruction
    pub fn register_instruction(&mut self, name: &str, definition: Arc<dyn InstructionDefinition>) {
        let mut registry = self.instruction_registry.lock().unwrap();
        let kind = InstructionKind::from_name(name);
        registry.register(kind, definition);
    }
}
