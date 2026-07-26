#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    pub verified: bool,
    pub compiled_size: i32,
    pub error_description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityFixture {
    pub name: &'static str,
    pub source: &'static str,
    pub expected_size: Option<i32>,
    pub expected_error_contains: Option<&'static str>,
}

static COMPATIBILITY_FIXTURES: &[CompatibilityFixture] = &[
    CompatibilityFixture {
        name: "default_program",
        source: "move(1);\nmine();",
        expected_size: Some(4),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "simple_while_comment",
        source: "// start\nwhile (mine());",
        expected_size: Some(2),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "all_robot_actions",
        source: "move(1); rotate(90); mine(); ore(0); dump(1); time();",
        expected_size: Some(11),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "seed_ai_1",
        source: "move(1.5); while (mine());",
        expected_size: Some(5),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "seed_ai_2",
        source: "if (move(1.5) >= 1) { while (mine()); } else { move(-1); rotate(20); }",
        expected_size: Some(12),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "seed_ai_3",
        source: "int rot = 0; while (true) { if (rot) { if (rot <= 90) { rotate(rot); } rot = rot - 10; } if (move(1.5) < 1) { move(-1); rotate(24); } while (mine()) { rot = 100; } }",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "flow_control",
        source: "if (true) { move(1); } else { rotate(90); }\ndo { mine(); } while (time() > 0);",
        expected_size: Some(12),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "while_boolean_expression",
        source: "while (!mine() && (move(2)>0.1)) { dump(0); }",
        expected_size: Some(10),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "const_and_increment",
        source: "const int limit = 3; int count = 0; while (count < limit) { count++; }",
        expected_size: Some(11),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "robot_properties",
        source: "if (robot.miningSpeed > 0) { move(robot.forwardSpeed); }",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "block_scope_reuse",
        source: "{ int value = 1; value++; } { int value = 2; --value; }",
        expected_size: Some(11),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "operator_precedence",
        source: "int mined = mine();\nif (mined > 0) { dump(1 + 2 * 3); }",
        expected_size: Some(13),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "balance_pool_1",
        source: include_str!("fixture_sources/balance_pool_1.rm"),
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "balance_pool_3",
        source: include_str!("fixture_sources/balance_pool_3.rm"),
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "invalid_missing_close_paren",
        source: "move(1",
        expected_size: None,
        expected_error_contains: Some("')' expected"),
    },
    CompatibilityFixture {
        name: "invalid_duplicate_variable",
        source: "int x = 1; int x = 2;",
        expected_size: None,
        expected_error_contains: Some("Duplicate variable declaration"),
    },
    CompatibilityFixture {
        name: "invalid_missing_semicolon",
        source: "move(1) mine();",
        expected_size: None,
        expected_error_contains: Some("Missing ';'"),
    },
    CompatibilityFixture {
        name: "invalid_const_missing_type",
        source: "const x = 1;",
        expected_size: None,
        expected_error_contains: Some("Variable type expected"),
    },
    CompatibilityFixture {
        name: "invalid_const_missing_value",
        source: "const int x;",
        expected_size: None,
        expected_error_contains: Some("const variables must be assigned"),
    },
    CompatibilityFixture {
        name: "invalid_const_assignment",
        source: "const int x = 1; x = 2;",
        expected_size: None,
        expected_error_contains: Some("const variable cannot be changed"),
    },
    CompatibilityFixture {
        name: "invalid_const_compound_assignment",
        source: "const int x = 1; x += 1;",
        expected_size: None,
        expected_error_contains: Some("const variable cannot be changed"),
    },
    CompatibilityFixture {
        name: "invalid_do_without_while",
        source: "do { mine(); }",
        expected_size: None,
        expected_error_contains: Some("'while' expected"),
    },
    CompatibilityFixture {
        name: "invalid_robot_property_assignment",
        source: "robot.forwardSpeed = 5;",
        expected_size: None,
        expected_error_contains: Some("Robot properties cannot be changed"),
    },
    CompatibilityFixture {
        name: "invalid_robot_property_increment",
        source: "robot.forwardSpeed++;",
        expected_size: None,
        expected_error_contains: Some("Robot properties cannot be changed"),
    },
    CompatibilityFixture {
        name: "invalid_robot_xpos_assignment",
        source: "robot.xPos = 1;",
        expected_size: None,
        expected_error_contains: Some("Robot properties cannot be changed"),
    },
    CompatibilityFixture {
        name: "invalid_robot_orientation_increment",
        source: "robot.orientation++;",
        expected_size: None,
        expected_error_contains: Some("Robot properties cannot be changed"),
    },
    CompatibilityFixture {
        name: "invalid_robot_property_unknown",
        source: "move(robot.foo);",
        expected_size: None,
        expected_error_contains: Some("Unknown robot property"),
    },
    CompatibilityFixture {
        name: "invalid_bare_pre_increment",
        source: "++;",
        expected_size: None,
        expected_error_contains: Some("Variable expected"),
    },
    CompatibilityFixture {
        name: "invalid_bare_pre_decrement",
        source: "--;",
        expected_size: None,
        expected_error_contains: Some("Variable expected"),
    },
    CompatibilityFixture {
        name: "invalid_empty_robot_property",
        source: "move(robot.);",
        expected_size: None,
        expected_error_contains: Some("Robot property expected"),
    },
    CompatibilityFixture {
        name: "pre_and_post_decrement",
        source: "int value = 3; --value; value--;",
        expected_size: Some(7),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "dynamic_dump_expression",
        source: "int slot = 1; dump(slot);",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "invalid_while_missing_paren",
        source: "while mine();",
        expected_size: None,
        expected_error_contains: Some("'(' expected"),
    },
    CompatibilityFixture {
        name: "invalid_do_missing_block",
        source: "do move(1); while (true);",
        expected_size: None,
        expected_error_contains: Some("'{' expected"),
    },
    CompatibilityFixture {
        name: "invalid_unknown_increment",
        source: "++unknown;",
        expected_size: None,
        expected_error_contains: Some("Variable expected"),
    },
    CompatibilityFixture {
        name: "scan_calls",
        source: "scan(); scan(90); oreDistance(); oreType();",
        expected_size: Some(6),
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "scan_search_loop",
        source: "scan(); while (oreType() == 0) { move(1); scan(); }",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "else_if_scan_chain",
        source: "scan(); if (oreType() > 0) {} else { scan(30); if (oreType() > 0) { rotate(30); } else { rotate(-30); } }",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "do_while_with_dump",
        source: "do { mine(); dump(0); } while (time() > 0);",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "do_while_mine",
        source: "do { mine(); } while (time() > 0);",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "scan_then_mine",
        source: "scan(); while (oreType() == 0) { move(1); scan(); } mine();",
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "ore_seeker_80x80",
        source: include_str!("fixture_sources/ore_seeker_80x80.rm"),
        expected_size: None,
        expected_error_contains: None,
    },
    CompatibilityFixture {
        name: "invalid_scan_missing_paren",
        source: "scan(90",
        expected_size: None,
        expected_error_contains: Some("')' expected"),
    },
];

pub fn compatibility_fixtures() -> &'static [CompatibilityFixture] {
    COMPATIBILITY_FIXTURES
}

pub fn compatibility_fixture(name: &str) -> Option<&'static CompatibilityFixture> {
    compatibility_fixtures()
        .iter()
        .find(|fixture| fixture.name == name)
}

pub fn compatibility_fixture_source(name: &str) -> &'static str {
    compatibility_fixture(name)
        .unwrap_or_else(|| panic!("unknown compatibility fixture: {name}"))
        .source
}

#[cfg(test)]
pub fn compatibility_fixtures_with_expected_size()
-> impl Iterator<Item = &'static CompatibilityFixture> {
    compatibility_fixtures()
        .iter()
        .filter(|fixture| fixture.expected_size.is_some())
}
