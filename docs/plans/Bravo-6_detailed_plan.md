Phase P1: Table Mode Support

     Goal: Handle table HTML correctly (extremely common in web pages)

     Files Modified

     - crates/rustkit-html/src/tree_builder.rs
     - crates/rustkit-html/src/lib.rs (TreeSink additions)

     New Insertion Modes

     enum InsertionMode {
         // ... existing ...
         InTable,        // After <table>
         InTableText,    // Whitespace in table context
         InCaption,      // Inside <caption>
         InColumnGroup,  // Inside <colgroup>
         InTableBody,    // Inside <tbody>/<thead>/<tfoot>
         InRow,          // Inside <tr>
         InCell,         // Inside <td>/<th>
     }

     Tasks

     | ID   | Task               | Complexity | Description                                   |
     |------|--------------------|------------|-----------------------------------------------|
     | P1.1 | InTable mode       | L          | Handle <table> context, foster parenting flag |
     | P1.2 | InTableBody mode   | M          | <tbody>, <thead>, <tfoot>                     |
     | P1.3 | InRow mode         | M          | <tr> element handling                         |
     | P1.4 | InCell mode        | M          | <td>, <th> cells                              |
     | P1.5 | InCaption mode     | S          | <caption> element                             |
     | P1.6 | InColumnGroup mode | S          | <colgroup>, <col>                             |
     | P1.7 | InTableText mode   | M          | Whitespace accumulation                       |
     | P1.8 | Foster parenting   | XL         | Move misnested content before table           |

     Foster Parenting Algorithm

     When content appears in table context that shouldn't be there:
     <table><p>text</p><tr><td>cell</td></tr></table>
     The <p> is "foster parented" to just before <table>:
     <p>text</p><table><tbody><tr><td>cell</td></tr></tbody></table>

     TreeSink Additions

     fn insert_element_at(&mut self, position: InsertionPosition, element: Self::NodeId);
     fn foster_parent_location(&self) -> InsertionPosition;

     Tests Required

     test_simple_table()           // <table><tr><td>cell</td></tr></table>
     test_nested_tables()          // Tables inside table cells
     test_table_foster_parenting() // <table><p>text</p><tr>
     test_implicit_tbody()         // <table><tr> implies <tbody>
     test_table_cell_scope()       // Scope calculations for cells

     ---
     Phase P2: Formatting Recovery (Adoption Agency Algorithm)

     Goal: Handle misnested formatting elements correctly

     The Problem

     <p><b><i>text</b></i></p>
     Should produce:
     <p><b><i>text</i></b><i></i></p>

     Tasks

     | ID   | Task                                     | Complexity | Description                                    |
     |------|------------------------------------------|------------|------------------------------------------------|
     | P2.1 | Active formatting elements list          | L          | Track <b>, <i>, <a>, <font>, etc. with markers |
     | P2.2 | reconstruct_active_formatting_elements() | L          | Reopen formatting after block elements         |
     | P2.3 | Adoption Agency Algorithm                | XL         | The core misnesting recovery algorithm         |
     | P2.4 | TreeSink methods for AAA                 | M          | reparent_children(), remove_from_parent()      |
     | P2.5 | Scope variant functions                  | M          | button_scope, list_scope, table_scope          |

     Adoption Agency Algorithm (Simplified)

     1. If current node is the formatting element, pop it and return
     2. Find "furthest block" - deepest block element after formatting element
     3. If no furthest block, pop everything up to formatting element
     4. Otherwise:
        a. Let "common ancestor" = element before formatting element
        b. Let "bookmark" = position of formatting element in active list
        c. Clone formatting element
        d. Move descendants of furthest block into clone
        e. Insert clone as child of furthest block
        f. Remove formatting element from tree
        g. Insert furthest block after common ancestor
     5. Repeat up to 8 times

     Formatting Elements

     const FORMATTING_ELEMENTS: &[&str] = &[
         "a", "b", "big", "code", "em", "font", "i", "nobr",
         "s", "small", "strike", "strong", "tt", "u"
     ];

     Tests Required

     test_misnested_bold_italic()     // <b><i></b></i>
     test_adoption_agency_limit()     // Max 8 iterations
     test_formatting_across_blocks()  // <b><p></b></p>
     test_anchor_adoption()           // <a><div></a></div>
     test_reconstruct_formatting()    // <b>bold<p>text</p>more</b>

     ---
     Phase P3: Complete Tokenizer States

     Goal: Full HTML5 tokenizer state machine (80+ states)

     New State Groups

     RCDATA Sub-states (3):
     - RCDATALessThanSign
     - RCDATAEndTagOpen
     - RCDATAEndTagName

     RAWTEXT Sub-states (3):
     - RAWTEXTLessThanSign
     - RAWTEXTEndTagOpen
     - RAWTEXTEndTagName

     ScriptData Sub-states (17):
     - ScriptDataLessThanSign, ScriptDataEndTagOpen, ScriptDataEndTagName
     - ScriptDataEscapeStart, ScriptDataEscapeStartDash
     - ScriptDataEscaped, ScriptDataEscapedDash, ScriptDataEscapedDashDash
     - ScriptDataEscapedLessThanSign, ScriptDataEscapedEndTagOpen, ScriptDataEscapedEndTagName
     - ScriptDataDoubleEscapeStart, ScriptDataDoubleEscaped
     - ScriptDataDoubleEscapedDash, ScriptDataDoubleEscapedDashDash
     - ScriptDataDoubleEscapedLessThanSign, ScriptDataDoubleEscapeEnd

     DOCTYPE Sub-states (13):
     - BeforeDoctypeName, AfterDoctypePublicKeyword
     - BeforeDoctypePublicIdentifier, DoctypePublicIdentifierDoubleQuoted
     - DoctypePublicIdentifierSingleQuoted, AfterDoctypePublicIdentifier
     - BetweenDoctypePublicAndSystemIdentifiers
     - AfterDoctypeSystemKeyword, BeforeDoctypeSystemIdentifier
     - DoctypeSystemIdentifierDoubleQuoted, DoctypeSystemIdentifierSingleQuoted
     - AfterDoctypeSystemIdentifier, BogusDoctype

     Character Reference States (9):
     - CharacterReference, NamedCharacterReference
     - AmbiguousAmpersand, NumericCharacterReference
     - HexadecimalCharacterReferenceStart, DecimalCharacterReferenceStart
     - HexadecimalCharacterReference, DecimalCharacterReference
     - NumericCharacterReferenceEnd

     Other (4):
     - CDATASection, CDATASectionBracket, CDATASectionEnd
     - PlainText

     Tasks

     | ID   | Task                       | Complexity | Description                         |
     |------|----------------------------|------------|-------------------------------------|
     | P3.1 | RCDATA sub-states          | M          | End tag detection in textarea/title |
     | P3.2 | RAWTEXT sub-states         | M          | End tag detection in style/xmp/etc  |
     | P3.3 | ScriptData basic           | L          | Basic script end tag detection      |
     | P3.4 | ScriptData escaped         | XL         | 17 states for <!-- inside scripts   |
     | P3.5 | DOCTYPE sub-states         | L          | PUBLIC/SYSTEM ID parsing            |
     | P3.6 | Character reference states | L          | Proper entity state machine         |
     | P3.7 | CDATA section states       | M          | For SVG/MathML content              |
     | P3.8 | PlainText state            | S          | Rarely used but spec-required       |

     ---
     Phase P4: Entity Expansion

     Goal: Complete HTML5 entity table (2,231 named entities)

     Current State

     - 35 entities in entities.rs HashMap
     - Missing: 2,196 entities

     Implementation Strategy

     Option A: Build-time Generation (Recommended)
     // build.rs
     fn main() {
         // Download/parse https://html.spec.whatwg.org/entities.json
         // Generate static phf_map! for O(1) lookup
     }

     Option B: Lazy Static HashMap
     lazy_static! {
         static ref ENTITIES: HashMap<&'static str, &'static str> = {
             include!(concat!(env!("OUT_DIR"), "/entities.rs"))
         };
     }

     Tasks

     | ID   | Task                         | Complexity | Description                              |
     |------|------------------------------|------------|------------------------------------------|
     | P4.1 | Generate entity table        | M          | All 2,231 from HTML5 spec                |
     | P4.2 | Legacy entity handling       | M          | Entities without semicolon in attributes |
     | P4.3 | Numeric reference validation | S          | Surrogate pairs, replacement char        |
     | P4.4 | Optimize lookup              | M          | phf crate for perfect hashing            |

     Legacy Entity Rules

     - In attributes: &copy (no semicolon) → © only if NOT followed by alphanumeric
     - In text: &copy → &copy (semicolon required)

     ---
     Phase P5: Select and Template Modes

     New Insertion Modes

     InSelect,          // Inside <select>
     InSelectInTable,   // <select> inside table
     InTemplate,        // Inside <template>
     InFrameset,        // Legacy <frameset>
     AfterFrameset,     // After </frameset>

     Template Element Special Handling

     - Template contents are in a separate DocumentFragment
     - Template stack separate from open elements
     - Modes reset when entering/exiting template

     Tasks

     | ID   | Task                 | Complexity |
     |------|----------------------|------------|
     | P5.1 | InSelect mode        | M          |
     | P5.2 | InSelectInTable mode | M          |
     | P5.3 | InTemplate mode      | L          |
     | P5.4 | Template stack       | M          |
     | P5.5 | InFrameset mode      | S          |
     | P5.6 | AfterFrameset mode   | S          |

     ---
     Phase P6: Fragment Parsing

     Goal: Support innerHTML-style parsing

     API Addition

     /// Parse HTML fragment in context of an element
     pub fn parse_fragment<S: TreeSink>(
         html: &str,
         context: &str,  // e.g., "div", "table", "select"
         sink: S
     ) -> ParseResult<S>

     Context Element Rules

     | Context         | Initial Mode | Initial Stack              |
     |-----------------|--------------|----------------------------|
     | html            | BeforeHead   | [html]                     |
     | head            | InHead       | [html, head]               |
     | body, div, etc. | InBody       | [html, body]               |
     | table           | InTable      | [html, body, table]        |
     | select          | InSelect     | [html, body, select]       |
     | script, style   | Text         | [html, head, script/style] |

     Tasks

     | ID   | Task                    | Complexity |
     |------|-------------------------|------------|
     | P6.1 | parse_fragment() API    | M          |
     | P6.2 | Context initialization  | M          |
     | P6.3 | Quirks mode inheritance | S          |
     | P6.4 | Tokenizer state init    | S          |

     ---
     Phase P7: Test Infrastructure

     Goal: Comprehensive test coverage with html5lib-tests

     html5lib-tests Integration

     Repository: https://github.com/html5lib/html5lib-tests

     Test Categories:
     - tree-construction/ - 2,700+ tree building tests
     - tokenizer/ - 3,000+ tokenizer tests
     - entities/ - Entity decoding tests

     Test Runner Implementation

     // tests/html5lib_runner.rs
     struct Html5libTestRunner {
         test_dir: PathBuf,
         expected_failures: HashSet<String>,
     }

     impl Html5libTestRunner {
         fn run_tree_construction_tests(&self) -> TestSummary;
         fn run_tokenizer_tests(&self) -> TestSummary;
     }

     Tasks

     | ID   | Task                           | Complexity |
     |------|--------------------------------|------------|
     | P7.1 | Tree construction test runner  | XL         |
     | P7.2 | Tokenizer test runner          | L          |
     | P7.3 | Regression corpus (100+ cases) | M          |
     | P7.4 | Fuzzing harness                | M          |
     | P7.5 | Performance benchmarks         | S          |

     Target Metrics

     - Tree construction: 95%+ pass rate
     - Tokenizer: 98%+ pass rate
     - Zero regressions on corpus

     ---
     Phase P8: TreeSink Enhancement

     New TreeSink Methods

     pub trait TreeSink {
         // ... existing ...

         // Error reporting
         fn parse_error(&mut self, message: &str);

         // Quirks mode
         fn set_quirks_mode(&mut self, mode: QuirksMode);

         // Advanced insertion
         fn insert_before(&mut self, parent: Self::NodeId, reference: Self::NodeId, new: Self::NodeId);
         fn insert_after(&mut self, reference: Self::NodeId, new: Self::NodeId);

         // Reparenting
         fn reparent_children(&mut self, from: Self::NodeId, to: Self::NodeId);
         fn remove_from_parent(&mut self, node: Self::NodeId);

         // Template support
         fn template_contents(&self, template: Self::NodeId) -> Self::NodeId;

         // Fragment parsing
         fn is_fragment_context(&self) -> bool;
     }

     ---
     Summary: Files to Modify

     | File                                    | Changes                              |
     |-----------------------------------------|--------------------------------------|
     | crates/rustkit-html/src/tokenizer.rs    | Fix bugs, add ~56 states             |
     | crates/rustkit-html/src/tree_builder.rs | Add 15 modes, AAA, foster parenting  |
     | crates/rustkit-html/src/entities.rs     | Expand to 2,231 entities             |
     | crates/rustkit-html/src/lib.rs          | TreeSink enhancements, fragment API  |
     | crates/rustkit-html/tests/              | html5lib-tests, corpus, fuzzing      |
     | crates/rustkit-dom/src/lib.rs           | Update DocumentSink for new TreeSink |

     Success Metrics

     | Metric                   | Current | Target |
     |--------------------------|---------|--------|
     | Tokenizer states         | 24      | 80+    |
     | Tree builder modes       | 8       | 23     |
     | Named entities           | 35      | 2,231  |
     | html5lib tree tests      | 0%      | 95%+   |
     | html5lib tokenizer tests | 0%      | 98%+   |

     Dependency Order

     P0 (Bug Fixes) ─────────────────────────────────┐
            │                                         │
            v                                         │
     P1 (Tables) ──────> P2 (AAA) ──────> P5 (Select)│
            │                │                        │
            v                v                        v
     P3 (Tokenizer) ────> P4 (Entities) ────> P6 (Fragment)
                                                      │
                                                      v
                                              P7 (Tests)
                                                      │
                                                      v
                                              P8 (TreeSink)