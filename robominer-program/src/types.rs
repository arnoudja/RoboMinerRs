use std::error::Error;
use std::fmt::{self, Display};

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
            .map(|statement| statement.source_line)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutableAction {
    Move(f64),
    Rotate(f64),
    Mine,
    Dump(i32),
    StartScan(f64),
    AwaitScanResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableStatement {
    pub source_line: u16,
    pub kind: ExecutableStatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableStatementKind {
    Action(ExecutableAction),
    DynamicAction(ExecutableActionExpression),
    Sequence(Vec<ExecutableStatement>),
    Declare {
        name: String,
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
    pub fn at(source_line: u16, kind: ExecutableStatementKind) -> Self {
        Self { source_line, kind }
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
pub enum ExecutableExpression {
    Number(f64),
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
            Self::MaxCycles => "maxCycles",
            Self::MiningSpeed => "miningSpeed",
            Self::CpuSpeed => "cpuSpeed",
            Self::Orientation => "orientation",
            Self::XPos => "xPos",
            Self::YPos => "yPos",
        }
    }

    pub fn value(self, robot: &RobotProperties) -> Option<f64> {
        Some(match self {
            Self::ForwardSpeed => robot.forward_speed,
            Self::BackwardSpeed => robot.backward_speed,
            Self::RotateSpeed => robot.rotate_speed,
            Self::ScanTime => robot.scan_time,
            Self::ScanDistance => robot.scan_distance,
            Self::OreCap => robot.ore_cap,
            Self::MaxCycles => robot.max_cycles,
            Self::MiningSpeed => robot.mining_speed,
            Self::CpuSpeed => robot.cpu_speed,
            Self::Orientation => robot.orientation,
            Self::XPos => robot.x_pos,
            Self::YPos => robot.y_pos,
            Self::OreStored | Self::OreStoredA | Self::OreStoredB | Self::OreStoredC => {
                return None;
            }
        })
    }

    pub fn stored_ore_value(self, ore: &[i32; 10]) -> Option<f64> {
        Some(match self {
            Self::OreStored => ore.iter().sum::<i32>() as f64,
            Self::OreStoredA => ore.first().copied().unwrap_or(0) as f64,
            Self::OreStoredB => ore.get(1).copied().unwrap_or(0) as f64,
            Self::OreStoredC => ore.get(2).copied().unwrap_or(0) as f64,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RobotProperties {
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: f64,
    pub scan_time: f64,
    pub scan_distance: f64,
    pub ore_cap: f64,
    pub max_cycles: f64,
    pub mining_speed: f64,
    pub cpu_speed: f64,
    pub orientation: f64,
    pub x_pos: f64,
    pub y_pos: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionContext {
    pub time_left: i32,
    pub ore: [i32; 10],
    pub action_result: Option<f64>,
    pub scan_time: i32,
    pub scan_started: bool,
    pub scan_complete: bool,
    pub scan_distance: f64,
    pub scan_ore_type: f64,
    pub robot: RobotProperties,
}

impl ExecutionContext {
    pub fn from_runtime(time_left: i32, ore: [i32; 10], action_result: Option<f64>) -> Self {
        Self {
            time_left,
            ore,
            action_result,
            scan_time: 0,
            scan_started: false,
            scan_complete: false,
            scan_distance: -1.0,
            scan_ore_type: 0.0,
            robot: RobotProperties::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgramStep {
    Cpu,
    Action(ExecutableAction),
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    message: String,
}

impl CompileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CompileError {}
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
