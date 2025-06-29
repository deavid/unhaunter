# Ghost List Tool - Design Document

## Overview

The Ghost List Tool is a command-line utility for analyzing ghost types, their evidence combinations, and optimal ghost sets for the Unhaunter game. This tool helps with game balance, difficulty tuning, and evidence analysis.

## Current Implementation Status (✅ = Complete, 🔄 = Partial, ❌ = Not Started)

### Implemented Features:
- ✅ **Basic CLI Structure**: Full clap-based CLI with subcommands
- ✅ **Evidence Filtering**: Global filters (--has-evidence, --missing-evidence, --has-all, --has-any)
- ✅ **Ghost List Display**: Table format working
- ✅ **Stats Command**: Shows evidence distribution and ghost counts
- ✅ **Set Commands Structure**: test-set, analyze-set, complete-set, validate-set subcommands
- 🔄 **Set Completion Logic**: Basic implementation exists, needs refinement
- 🔄 **Set Validation**: Uniqueness validation working, needs advanced features
- ❌ **JSON/CSV Output**: Stub implementations only
- ❌ **Advanced Set Analysis**: Beyond basic evidence distribution

### Code Architecture:
```
/tools/ghost_list/src/
├── main.rs                    # Entry point (minimal, just calls CLI)
├── lib.rs                     # Module exports
├── cli/mod.rs                 # CLI structure and command dispatch
├── filtering/
│   ├── mod.rs                 # Evidence filtering logic
│   └── evidence_parser.rs     # Parse evidence strings
├── analysis/
│   ├── mod.rs                 # Analysis coordination
│   └── stats.rs              # Evidence statistics
├── export/
│   ├── mod.rs                 # Output format coordination
│   ├── table.rs              # Table output (working)
│   ├── json.rs               # JSON output (stub)
│   └── csv.rs                # CSV output (stub)
├── sets/
│   ├── mod.rs                # Set analysis commands
│   ├── completion.rs         # Set completion logic
│   ├── validation.rs         # Set uniqueness validation
│   ├── optimization.rs       # Optimization (stub)
│   └── comparison.rs         # Set comparison (stub)
└── utils/
    ├── mod.rs                # Utility exports
    └── ghost_parser.rs       # Parse ghost name lists
```

### Dependencies:
- ✅ clap (CLI framework)
- ✅ itertools (for evidence combinations)
- ✅ uncore (ghost/evidence types)
- ✅ enum-iterator (iterate over ghost/evidence types)

### Working Commands:
```bash
# These work fully:
ghost_list                                    # List all ghosts (table format)
ghost_list --has-evidence "Freezing Temps"   # Filter by evidence
ghost_list stats                              # Evidence statistics

# These work partially:
ghost_list test-set "Caoilte,Ceara"         # Basic ghost parsing + display
ghost_list complete-set "Caoilte,Ceara"     # Find completing ghosts (basic)
ghost_list validate-set "Caoilte,Ceara"     # Uniqueness validation

# These don't work yet:
ghost_list --format json                     # JSON output (stub)
ghost_list --format csv                      # CSV output (stub)
```

## Planned Features (Full Specification)

### 1. Evidence Filtering ✅ IMPLEMENTED
Filter ghosts based on evidence requirements.

**Commands:**
```bash
# Find ghosts with specific evidence
ghost_list --has-evidence "Freezing Temps,EMF Level 5"
ghost_list --missing-evidence "UV Ectoplasm,Floating Orbs"

# Combination filters
ghost_list --has-evidence "Freezing Temps" --missing-evidence "UV Ectoplasm"

# Multiple evidence requirements (AND logic)
ghost_list --has-all "Freezing Temps,EMF Level 5"

# Any evidence match (OR logic)
ghost_list --has-any "Freezing Temps,EMF Level 5"
```

**Status**: ✅ Complete - All filtering options work correctly

### 2. Ghost Set Analysis 🔄 PARTIALLY IMPLEMENTED
Analyze and optimize ghost sets for gameplay.

**Commands:**
```bash
# Test existing ghost sets
ghost_list test-set "Caoilte,Ceara,Orla,Finvarra,Kappa"

# Analyze set balance and gaps
ghost_list analyze-set "Caoilte,Ceara,Orla,Finvarra,Kappa"

# Find what ghosts would complete a partial set
ghost_list complete-set "Caoilte,Ceara" --requires-evidence "Freezing Temps,EMF Level 5" --excludes-evidence "UV Ectoplasm,Floating Orbs"

# Validate if a set is uniquely identifiable
ghost_list validate-set "Ghost1,Ghost2,Ghost3" --min-evidence 2
```

**Status**:
- ✅ CLI structure complete
- ✅ Basic ghost parsing and display
- 🔄 `complete-set`: Basic implementation, needs refinement
- 🔄 `validate-set`: Uniqueness checking works, needs advanced features
- ❌ `test-set`: Stub implementation only
- ❌ `analyze-set`: Shows evidence distribution only

**Next Steps**: Enhance the analysis logic in each command

### 3. Optimal Set Generation ❌ NOT STARTED
Find optimal ghost sets based on criteria.

**Commands:**
```bash
# Find optimal ghost sets for specific evidence combinations
ghost_list find-sets --target-evidence "Freezing Temps,EMF Level 5,UV Ectoplasm" --size 5 --min-coverage 0.8

# Generate balanced sets
ghost_list optimize-set --size 6 --balance-factor 0.8 --max-overlap 2

# Find sets that maximize evidence diversity
ghost_list diverse-set --size 10 --min-evidence-coverage 3

# Generate sets for specific gameplay scenarios
ghost_list tutorial-set --beginner-friendly --size 4
```

**Status**: ❌ Not implemented - Need to integrate ghost_setfinder.rs logic

### 4. Evidence Analysis 🔄 PARTIALLY IMPLEMENTED
Deep analysis of evidence patterns and conflicts.

**Commands:**
```bash
# Show evidence distribution and statistics
ghost_list stats --evidence-distribution  # ✅ WORKING
ghost_list stats --ghost-count            # ✅ WORKING
ghost_list stats --balance-report         # ❌ NOT IMPLEMENTED

# Find evidence conflicts and overlaps
ghost_list conflicts --evidence "Freezing Temps,EMF Level 5"    # ❌ NOT IMPLEMENTED
ghost_list conflicts --show-all                                 # ❌ NOT IMPLEMENTED

# Show unique evidence combinations
ghost_list unique-combinations                                   # ❌ NOT IMPLEMENTED
ghost_list unique-combinations --min-evidence 2 --max-evidence 3

# Evidence correlation analysis
ghost_list correlate --evidence "Freezing Temps" --with "EMF Level 5"  # ❌ NOT IMPLEMENTED
```

**Status**:
- ✅ Basic stats command works
- ❌ Advanced analysis commands not implemented

### 5. Set Comparison Tools ❌ NOT STARTED
Compare multiple ghost sets and analyze differences.

**Commands:**
```bash
# Compare multiple ghost sets
ghost_list compare-sets "TmpEMF:LadyInWhite,BrownLady" "TmpEMFUVOrbs:Caoilte,Ceara,Orla,Finvarra,Kappa"

# Show set overlap analysis
ghost_list overlap-analysis --sets "Set1:Ghost1,Ghost2" "Set2:Ghost3,Ghost4"

# Merge set analysis
ghost_list merge-sets "Set1:Ghost1,Ghost2" "Set2:Ghost3,Ghost4" --optimize

# Diff between sets
ghost_list diff-sets "OldSet:Ghost1,Ghost2" "NewSet:Ghost1,Ghost3"
```

**Status**: ❌ Not implemented

### 6. Advanced Features 🔄 PARTIALLY IMPLEMENTED

#### 6.1 Output Formats
```bash
# Different output formats
ghost_list --format json     # ❌ Stub only
ghost_list --format csv      # ❌ Stub only
ghost_list --format table    # ✅ Working (default)
ghost_list --format yaml     # ❌ Not planned yet
```

**Status**: Only table format works, JSON/CSV are stubs

#### 6.2 Validation Tools ❌ NOT STARTED
```bash
# Validate game balance
ghost_list validate --all-sets
ghost_list validate --evidence-balance
ghost_list validate --uniqueness

# Check for design issues
ghost_list lint --check-duplicates
ghost_list lint --check-balance
ghost_list lint --check-coverage
```

#### 6.3 Export/Import ❌ NOT STARTED
```bash
# Export sets for external analysis
ghost_list export --set "MySet" --format json > myset.json

# Import and test external sets
ghost_list import-set myset.json --validate

# Generate configuration files
ghost_list generate-config --difficulty-preset easy > easy_config.ron
```

## Implementation Roadmap & Next Steps

### IMMEDIATE PRIORITIES (Phase 2A - Set Analysis Completion):

#### 1. Complete JSON/CSV Output (Easy Wins - 30 min)
**Files to edit:**
- `src/export/json.rs` - implement `show_ghost_json()`
- `src/export/csv.rs` - implement `show_ghost_csv()`

**Implementation notes:**
- JSON: Create simple struct with ghost name + evidence list, use serde_json
- CSV: Headers = "Ghost,Evidence1,Evidence2,Evidence3" format
- Add serde dependency to Cargo.toml if needed

#### 2. Enhance Set Analysis Commands (1-2 hours)
**Files to edit:**
- `src/sets/mod.rs` - expand `analyze_set()` and `test_set()`
- `src/sets/validation.rs` - add advanced validation features

**Missing features to implement:**
- **test_set**: Add conflict detection, balance scoring, uniqueness preview
- **analyze_set**: Add gap analysis, recommendation engine, balance metrics
- **validate_set**: Add minimum evidence validation, conflict reporting

#### 3. Integrate ghost_setfinder.rs Logic (1-2 hours)
**Files to check:**
- `/home/deavid/git/rust/unhaunter/tools/ghost_radio/src/ghost_setfinder.rs` (existing logic)
- Integrate optimal set generation algorithms into `src/sets/optimization.rs`

### PHASE 2B - Advanced Features:

#### 4. Evidence Analysis Commands (2-3 hours)
**New commands to implement:**
```bash
ghost_list conflicts --evidence "Freezing Temps,EMF Level 5"
ghost_list unique-combinations --min-evidence 2 --max-evidence 3
ghost_list correlate --evidence "Freezing Temps" --with "EMF Level 5"
```

**Files to create/edit:**
- `src/analysis/conflicts.rs` - evidence conflict detection
- `src/analysis/combinations.rs` - unique evidence combination analysis
- `src/analysis/correlation.rs` - evidence correlation analysis
- Update `src/cli/mod.rs` with new subcommands

#### 5. Set Generation Commands (3-4 hours)
**New commands to implement:**
```bash
ghost_list find-sets --target-evidence "Freezing Temps,EMF Level 5" --size 5
ghost_list optimize-set --size 6 --balance-factor 0.8
ghost_list diverse-set --size 10 --min-evidence-coverage 3
```

**Files to edit:**
- `src/sets/optimization.rs` - implement set generation algorithms
- `src/cli/mod.rs` - add new subcommands
- Integrate with existing ghost_setfinder logic

### PHASE 3 - Comparison & Advanced Features:

#### 6. Set Comparison Tools
```bash
ghost_list compare-sets "Set1:Ghost1,Ghost2" "Set2:Ghost3,Ghost4"
ghost_list overlap-analysis --sets "Set1:Ghost1,Ghost2" "Set2:Ghost3,Ghost4"
```

#### 7. Export/Import & Configuration
```bash
ghost_list export --set "MySet" --format json > myset.json
ghost_list generate-config --difficulty-preset easy > easy_config.ron
```

## Technical Context for Next Developer

### Key Integration Points:

1. **Ghost Types**: Use `uncore::types::ghost::types::GhostType` enum
   - Access via `enum_iterator::all::<GhostType>()`
   - Each ghost has `.evidences()` method returning `Vec<Evidence>`
   - Each ghost has `.name()` method

2. **Evidence Types**: Use `uncore::types::evidence::Evidence` enum
   - Access via `enum_iterator::all::<Evidence>()`
   - Each evidence has `.name()` method

3. **Existing Algorithms**: Check `/tools/ghost_radio/src/ghost_setfinder.rs`
   - Contains set optimization logic that should be integrated
   - Look for functions like optimal set generation, balance scoring

4. **Error Handling Pattern**: Current code uses `eprintln!()` for errors
   - Consider upgrading to proper Result<T, E> returns
   - Add more robust validation and user feedback

5. **Output Formatting**: All commands should respect `--format` flag
   - Table format is fully implemented
   - JSON/CSV need implementation
   - Consider adding YAML support later

### Code Patterns:

1. **Ghost Parsing**: Use `crate::utils::parse_ghost_list(ghost_names: &str)`
   - Handles comma-separated ghost names
   - Returns `Vec<GhostType>` with invalid names filtered out
   - Provides user feedback for invalid names

2. **Evidence Parsing**: Use `crate::filtering::evidence_parser::parse_evidence_list()`
   - Handles comma-separated evidence names
   - Case-insensitive matching
   - Returns parsed evidence or errors

3. **CLI Command Pattern**:
```rust
// In src/cli/mod.rs Commands enum:
NewCommand {
    #[arg(help = "Description")]
    param: String,
    #[arg(long, help = "Optional flag")]
    flag: Option<String>,
},

// In execute() method:
Some(Commands::NewCommand { param, flag }) => {
    new_module::handle_command(param, flag.as_deref())
}
```

### Testing Commands:

```bash
# Basic functionality:
cargo run -- --help
cargo run -- stats
cargo run -- test-set "Caoilte,Ceara,Orla"

# Evidence filtering:
cargo run -- --has-evidence "Freezing Temps"
cargo run -- --missing-evidence "UV Ectoplasm"

# Set analysis:
cargo run -- complete-set "Caoilte,Ceara" --requires-evidence "Freezing Temps"
cargo run -- validate-set "Caoilte,Ceara,Orla" --min-evidence 2

# Output formats (JSON/CSV need implementation):
cargo run -- stats --format json
cargo run -- --format csv
```

### Quick Start for Next Developer:

1. **First 30 minutes**: Implement JSON/CSV output (easy wins)
   - Edit `src/export/json.rs` and `src/export/csv.rs`
   - Test with `cargo run -- --format json`

2. **Next 1-2 hours**: Enhance set analysis commands
   - Focus on `src/sets/mod.rs` - make `analyze_set()` more useful
   - Add actual logic beyond evidence distribution

3. **After that**: Check existing `ghost_setfinder.rs` and integrate
   - Look for existing optimization algorithms
   - Port them to `src/sets/optimization.rs`

4. **Then**: Add new analysis commands (conflicts, correlations, etc.)

### Known Issues & TODOs:

1. **Ghost Name Parsing**: Currently case-sensitive, consider fuzzy matching
2. **Evidence Name Parsing**: Works but could use better error messages
3. **Output Consistency**: Some commands print to stdout, others stderr
4. **Performance**: Should handle large ghost sets efficiently
5. **Documentation**: Add --help text improvements
6. **Testing**: No unit tests yet - consider adding them

## Technical Architecture

### CLI Structure
```rust
#[derive(Parser)]
#[command(name = "ghost_list")]
#[command(about = "Unhaunter ghost analysis and set optimization tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Global options
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,

    #[arg(long)]
    has_evidence: Option<String>,

    #[arg(long)]
    missing_evidence: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    TestSet { ghosts: String },
    AnalyzeSet { ghosts: String },
    CompleteSet {
        ghosts: String,
        #[arg(long)] requires_evidence: Option<String>,
        #[arg(long)] excludes_evidence: Option<String>,
    },
    FindSets {
        #[arg(long)] target_evidence: String,
        #[arg(long)] size: usize,
    },
    // ... more commands
}
```

### Core Modules
- `filtering.rs` - Evidence-based ghost filtering
- `set_analysis.rs` - Ghost set analysis and validation
- `optimization.rs` - Set generation and optimization
- `statistics.rs` - Evidence statistics and correlation
- `output.rs` - Formatting and export functionality
- `validation.rs` - Game balance validation

### Integration Points
- Use existing `ghost_setfinder.rs` logic for optimization
- Integrate with `GhostType` and `Evidence` enums
- Leverage existing test infrastructure

## Example Usage Scenarios & Expected Outputs

### Scenario 1: Fixing TmpEMFUVOrbs Set (Currently Working!)
```bash
# Find the missing ghost for TmpEMFUVOrbs set
$ ghost_list complete-set "Caoilte,Ceara,Orla,Finvarra,Kappa" \
  --requires-evidence "Freezing Temps,EMF Level 5" \
  --excludes-evidence "UV Ectoplasm,Floating Orbs"

# Expected output: Should suggest "Gray Man" or similar ghosts
# Currently works but needs refinement in candidate ranking
```

### Scenario 2: Creating Tutorial Sets (Partially Working)
```bash
# Step 1: Check uniqueness of a potential tutorial set
$ ghost_list validate-set "Caoilte,Ceara,Orla,Finvarra" --min-evidence 2

# Step 2: Generate a beginner-friendly set (NOT YET IMPLEMENTED)
$ ghost_list diverse-set --size 4 --min-evidence-coverage 6 --beginner-friendly

# Step 3: Test the resulting set (BASIC IMPLEMENTATION)
$ ghost_list test-set "Ghost1,Ghost2,Ghost3,Ghost4"
```

### Scenario 3: Game Balance Analysis (Partially Working)
```bash
# Check evidence distribution balance (WORKS)
$ ghost_list stats --evidence-distribution

# Find evidence combinations that appear too frequently (NOT IMPLEMENTED)
$ ghost_list correlate --threshold 0.8 --show-problematic

# Find all conflicts in current ghost database (NOT IMPLEMENTED)
$ ghost_list conflicts --show-all
```

### Scenario 4: Quick Ghost Lookup (Working)
```bash
# Find all ghosts with freezing temperatures
$ ghost_list --has-evidence "Freezing Temps"

# Find ghosts that DON'T have UV evidence (useful for certain scenarios)
$ ghost_list --missing-evidence "UV Ectoplasm"

# Export results as JSON for external tools (STUB ONLY)
$ ghost_list --has-evidence "Freezing Temps" --format json
```

## Key Integration Files & References

### Existing Code to Integrate:
1. **`/tools/ghost_radio/src/ghost_setfinder.rs`** - Contains optimization algorithms
   - Look for functions like `find_optimal_set()`, `calculate_balance_score()`
   - Port these to `src/sets/optimization.rs`

2. **Ghost/Evidence Data**: Available via uncore crate
   - `uncore::types::ghost::types::GhostType` - All ghost types
   - `uncore::types::evidence::Evidence` - All evidence types
   - Access via `enum_iterator::all::<GhostType>()`

### Critical Implementation Notes:

1. **Evidence Names**: Use exact strings from the codebase
   - "Freezing Temps", "EMF Level 5", "UV Ectoplasm", "Floating Orbs", etc.
   - Check `Evidence::name()` method for canonical names

2. **Ghost Names**: Use exact ghost names from GhostType enum
   - "Caoilte", "Ceara", "Orla", "Finvarra", "Kappa", "Gray Man", etc.
   - Parser currently case-sensitive, may want to improve

3. **Output Consistency**:
   - Use `println!()` for normal output
   - Use `eprintln!()` for errors/warnings
   - Respect `--format` flag in all commands

4. **Performance Expectations**:
   - Full ghost database analysis should complete in <1 second
   - Set operations should handle up to 20+ ghosts efficiently
   - Evidence combination analysis may be computationally intensive

## Success Criteria

When this tool is "complete", users should be able to:

1. **✅ DONE**: Filter ghosts by evidence and get clean output
2. **🔄 PARTIAL**: Analyze ghost sets for balance and conflicts
3. **❌ TODO**: Generate optimal ghost sets for different difficulties
4. **❌ TODO**: Export results in JSON/CSV for external analysis
5. **❌ TODO**: Find evidence conflicts and correlation patterns
6. **❌ TODO**: Compare different ghost set configurations

## Immediate Next Steps for Continuing Developer

### 30-Minute Quick Wins:
1. Implement JSON output in `src/export/json.rs`
2. Implement CSV output in `src/export/csv.rs`
3. Test with `cargo run -- --format json` and `cargo run -- --format csv`

### 1-2 Hour Tasks:
1. Enhance `test_set()` command to actually test for conflicts
2. Improve `analyze_set()` with gap analysis and recommendations
3. Check and integrate existing `ghost_setfinder.rs` algorithms

### 2-4 Hour Features:
1. Add evidence conflict detection commands
2. Add evidence correlation analysis
3. Add optimal set generation commands
4. Improve error handling and user feedback

### Testing Commands to Verify Progress:
```bash
# Test basic functionality
cargo run -- --help
cargo run -- stats
cargo run -- --has-evidence "Freezing Temps"

# Test set commands
cargo run -- test-set "Caoilte,Ceara,Orla"
cargo run -- validate-set "Caoilte,Ceara,Orla" --min-evidence 2
cargo run -- complete-set "Caoilte,Ceara"

# Test output formats
cargo run -- stats --format table
cargo run -- stats --format json
cargo run -- stats --format csv
```

The codebase is well-structured and ready for the next developer to jump in! 🚀
