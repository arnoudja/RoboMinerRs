use crate::compile_error::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableProgram {
    pub statements: Vec<ExecutableStatement>,
    pub actions: Vec<ExecutableAction>,
    pub requires_runtime: bool,
}

impl ExecutableProgram {
    pub fn statements(&self) -> &[ExecutableStatement] {
        &self.statements
    }

    pub fn actions(&self) -> &[ExecutableAction] {
        &self.actions
    }

    pub fn requires_runtime(&self) -> bool {
        self.requires_runtime
    }

    /// 1-based source line of the first top-level statement, if any.
    pub fn entry_source_line(&self) -> Option<u16> {
        self.statements
            .first()
            .map(|statement| statement.source_line())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutableAction {
    Move(f64),
    Rotate(f64),
    Mine,
    /// Dump cargo: `0` = all types; `1`/`2`/`3` = A/B/C (same slots as `robot.oreStoredA|B|C`).
    /// Prefer `dump()` / `dumpA()` / `dumpB()` / `dumpC()` in new programs; `dump(n)` remains for compatibility.
    Dump(i32),
    StartScan(f64),
    AwaitScanResult,
}

/// Location of a construct in the **displayed** program source.
///
/// Columns are 1-based, with `start_col` inclusive and `end_col` exclusive, which is
/// what the rally replay UI needs to highlight a range. They are measured in the source
/// the player edits: the compiler wraps that source in an implicit `{ ... }` block and
/// the wrapper is not counted.
///
/// `line == 0` means the location is unknown; `start_col == 0` means only the line is
/// known (AST nodes synthesised outside the parser, for example by GP mutation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceSpan {
    pub line: u16,
    pub start_col: u16,
    pub end_col: u16,
}

impl SourceSpan {
    pub const UNKNOWN: Self = Self {
        line: 0,
        start_col: 0,
        end_col: 0,
    };

    /// Span for a node whose line is known but whose columns are not.
    pub fn line_only(line: u16) -> Self {
        Self {
            line,
            start_col: 0,
            end_col: 0,
        }
    }

    pub fn is_known(self) -> bool {
        self.line != 0
    }

    pub fn has_columns(self) -> bool {
        self.line != 0 && self.start_col != 0 && self.end_col > self.start_col
    }

    /// Smallest span covering both operands. Spans on different lines cannot be
    /// represented, so the left one wins.
    pub fn join(self, other: Self) -> Self {
        if !self.is_known() {
            return other;
        }
        if !other.is_known() || self.line != other.line {
            return self;
        }
        if !self.has_columns() || !other.has_columns() {
            return Self::line_only(self.line);
        }
        Self {
            line: self.line,
            start_col: self.start_col.min(other.start_col),
            end_col: self.end_col.max(other.end_col),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableStatement {
    pub source_span: SourceSpan,
    pub kind: ExecutableStatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableStatementKind {
    Action(ExecutableAction),
    DynamicAction(ExecutableActionExpression),
    Sequence(Vec<ExecutableStatement>),
    Declare {
        name: String,
        value_type: ValueType,
        value: Option<ExecutableExpression>,
    },
    Assign {
        name: String,
        value: ExecutableExpression,
    },
    Expression(ExecutableExpression),
    If {
        condition: ExecutableExpression,
        true_body: Box<ExecutableStatement>,
        false_body: Option<Box<ExecutableStatement>>,
    },
    While {
        condition: ExecutableExpression,
        body: Option<Box<ExecutableStatement>>,
        is_do_while: bool,
    },
}

impl ExecutableStatement {
    pub fn at(source_span: SourceSpan, kind: ExecutableStatementKind) -> Self {
        Self { source_span, kind }
    }

    pub fn source_line(&self) -> u16 {
        self.source_span.line
    }

    pub fn requires_runtime(&self) -> bool {
        match &self.kind {
            ExecutableStatementKind::Action(action) => matches!(
                action,
                ExecutableAction::StartScan(_) | ExecutableAction::AwaitScanResult
            ),
            ExecutableStatementKind::DynamicAction(_) => true,
            ExecutableStatementKind::Sequence(statements) => {
                statements.iter().any(ExecutableStatement::requires_runtime)
            }
            ExecutableStatementKind::Declare { .. }
            | ExecutableStatementKind::Assign { .. }
            | ExecutableStatementKind::Expression(_) => true,
            ExecutableStatementKind::If { .. } | ExecutableStatementKind::While { .. } => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableActionExpression {
    Move(ExecutableExpression),
    Rotate(ExecutableExpression),
    Dump(ExecutableExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableExpression {
    pub span: SourceSpan,
    pub kind: ExecutableExpressionKind,
}

impl ExecutableExpression {
    pub fn new(span: SourceSpan, kind: ExecutableExpressionKind) -> Self {
        Self { span, kind }
    }

    /// Expression without source location, for nodes built outside the parser.
    pub fn unspanned(kind: ExecutableExpressionKind) -> Self {
        Self {
            span: SourceSpan::UNKNOWN,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableExpressionKind {
    Number(f64),
    Bool(bool),
    Variable(String),
    VariableUpdate {
        name: String,
        operator: VariableOperator,
    },
    UnaryNot(Box<ExecutableExpression>),
    Binary {
        operator: Operator,
        left: Box<ExecutableExpression>,
        right: Box<ExecutableExpression>,
    },
    Time,
    /// Deprecated cargo query (`ore(n)`). Prefer `robot.oreStored` / `robot.oreStoredA|B|C`.
    /// Kept so existing robot programs keep compiling and running.
    Ore(Box<ExecutableExpression>),
    Scan(Option<Box<ExecutableExpression>>),
    OreDistance,
    OreType,
    RobotProperty(RobotProperty),
    Move(Box<ExecutableExpression>),
    Rotate(Box<ExecutableExpression>),
    Dump(Box<ExecutableExpression>),
    Action(ExecutableAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotProperty {
    ForwardSpeed,
    BackwardSpeed,
    RotateSpeed,
    ScanTime,
    ScanDistance,
    OreCap,
    /// Total ore currently in the container (`ore(0)`).
    OreStored,
    /// Highest-quality ore currently in the container (`ore(1)`).
    OreStoredA,
    /// Medium-quality ore currently in the container (`ore(2)`).
    OreStoredB,
    /// Lowest-quality ore currently in the container (`ore(3)`).
    OreStoredC,
    /// Depot capacity for highest-quality ore (slot A).
    DepotSizeA,
    /// Depot capacity for medium-quality ore (slot B).
    DepotSizeB,
    /// Depot capacity for lowest-quality ore (slot C).
    DepotSizeC,
    /// Highest-quality ore currently stored in the depot (slot A).
    DepotStoredA,
    /// Medium-quality ore currently stored in the depot (slot B).
    DepotStoredB,
    /// Lowest-quality ore currently stored in the depot (slot C).
    DepotStoredC,
    MaxCycles,
    MiningSpeed,
    CpuSpeed,
    Orientation,
    XPos,
    YPos,
}

impl RobotProperty {
    pub fn from_name(name: &str, line: usize) -> Result<Self, CompileError> {
        match name {
            "forwardSpeed" => Ok(Self::ForwardSpeed),
            "backwardSpeed" => Ok(Self::BackwardSpeed),
            "rotateSpeed" => Ok(Self::RotateSpeed),
            "scanTime" => Ok(Self::ScanTime),
            "scanDistance" => Ok(Self::ScanDistance),
            "oreCap" => Ok(Self::OreCap),
            "oreStored" => Ok(Self::OreStored),
            "oreStoredA" => Ok(Self::OreStoredA),
            "oreStoredB" => Ok(Self::OreStoredB),
            "oreStoredC" => Ok(Self::OreStoredC),
            "depotSizeA" => Ok(Self::DepotSizeA),
            "depotSizeB" => Ok(Self::DepotSizeB),
            "depotSizeC" => Ok(Self::DepotSizeC),
            "depotStoredA" => Ok(Self::DepotStoredA),
            "depotStoredB" => Ok(Self::DepotStoredB),
            "depotStoredC" => Ok(Self::DepotStoredC),
            "maxCycles" => Ok(Self::MaxCycles),
            "miningSpeed" => Ok(Self::MiningSpeed),
            "cpuSpeed" => Ok(Self::CpuSpeed),
            "orientation" => Ok(Self::Orientation),
            "xPos" => Ok(Self::XPos),
            "yPos" => Ok(Self::YPos),
            other => Err(CompileError::new(format!(
                "Syntax error at line {line}. Unknown robot property '{other}'"
            ))),
        }
    }

    pub fn as_name(self) -> &'static str {
        match self {
            Self::ForwardSpeed => "forwardSpeed",
            Self::BackwardSpeed => "backwardSpeed",
            Self::RotateSpeed => "rotateSpeed",
            Self::ScanTime => "scanTime",
            Self::ScanDistance => "scanDistance",
            Self::OreCap => "oreCap",
            Self::OreStored => "oreStored",
            Self::OreStoredA => "oreStoredA",
            Self::OreStoredB => "oreStoredB",
            Self::OreStoredC => "oreStoredC",
            Self::DepotSizeA => "depotSizeA",
            Self::DepotSizeB => "depotSizeB",
            Self::DepotSizeC => "depotSizeC",
            Self::DepotStoredA => "depotStoredA",
            Self::DepotStoredB => "depotStoredB",
            Self::DepotStoredC => "depotStoredC",
            Self::MaxCycles => "maxCycles",
            Self::MiningSpeed => "miningSpeed",
            Self::CpuSpeed => "cpuSpeed",
            Self::Orientation => "orientation",
            Self::XPos => "xPos",
            Self::YPos => "yPos",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Bool,
    Int,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableOperator {
    None,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Undefined,
    Addition,
    Subtraction,
    Multiply,
    Division,
    Mod,
    Larger,
    Smaller,
    LargerEqual,
    SmallerEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

impl Operator {
    pub fn as_token(self) -> &'static str {
        match self {
            Self::Undefined => "",
            Self::Addition => "+",
            Self::Subtraction => "-",
            Self::Multiply => "*",
            Self::Division => "/",
            Self::Mod => "%",
            Self::Larger => ">",
            Self::Smaller => "<",
            Self::LargerEqual => ">=",
            Self::SmallerEqual => "<=",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::And => "&&",
            Self::Or => "||",
        }
    }

    pub fn priority(self) -> usize {
        match self {
            Self::Multiply | Self::Division | Self::Mod => 4,
            Self::Addition | Self::Subtraction => 3,
            Self::Larger
            | Self::Smaller
            | Self::LargerEqual
            | Self::SmallerEqual
            | Self::Equal
            | Self::NotEqual => 2,
            Self::And | Self::Or => 1,
            Self::Undefined => 0,
        }
    }
}
