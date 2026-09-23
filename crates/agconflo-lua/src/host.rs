//! The script host: one activation, performed by running its node type's script
//! in a Lua state made for it alone, with nothing to reach but the activation
//! and the context API, and under its limits.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

use agconflo_core::{Activation, Context, ContextType, IdSource, OutputRefusal};
use mlua::prelude::*;

use crate::behaviours::Script;

/// What one activation's script may spend.
///
/// Counted in instructions executed and bytes allocated rather than in time
/// (`DEC_LIMITS_NOT_TIME`): the same script stops at the same point on every
/// machine, and a slow machine is not a runaway script. Each activation has its
/// own, since nothing else is shared between activations either.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Instructions one activation's script may execute. Counted in steps of a
    /// thousand, so a script is stopped within a thousand instructions of it.
    pub instructions: u64,
    /// Bytes one activation's Lua state may hold, the state's own included.
    pub memory: usize,
}

impl Default for Limits {
    /// Ten million instructions and 64 MiB: a few tens of milliseconds of work,
    /// and several thousand times what an empty state holds. Neither number is
    /// a requirement; both are room for a script that assembles text.
    fn default() -> Self {
        Self {
            instructions: 10_000_000,
            memory: 64 << 20,
        }
    }
}

/// How an activation's script failed.
///
/// Carried as the run's failure, so a caller learns which of these it was as a
/// value (`STKH_TYPED_FAILURE`). A limit is never reported as a script error nor
/// the other way round: both limits reach a script as Lua errors, and telling
/// them apart is this type's reason to exist.
///
/// `#[non_exhaustive]`: what a script can do wrong grows with what it can do.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScriptFailure {
    /// The script raised an error, or one was raised in something it called
    /// (`CREQ_HOST_ERROR_CARRIED`).
    Raised {
        /// The error as Lua reports it, with the document, the line and the
        /// traceback. An error raised with something other than a string has no
        /// message worth the name: a table renders as its address.
        message: String,
    },
    /// The script returned something other than exactly one context
    /// (`CREQ_HOST_ONE_CONTEXT`).
    NotOneContext {
        /// What it returned: `nothing`, a Lua type's name, or how many values.
        found: String,
    },
    /// The script executed more instructions than its limit
    /// (`CREQ_HOST_INSTRUCTION_LIMIT`).
    InstructionLimit,
    /// The script allocated more memory than its limit
    /// (`CREQ_HOST_MEMORY_LIMIT`).
    MemoryLimit,
    /// The script returned a context the run refused
    /// (`CREQ_HOST_OUTPUT_REFUSAL_CARRIED`), and this is the run's refusal.
    OutputRefused(OutputRefusal),
}

impl fmt::Display for ScriptFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raised { message } => write!(f, "the script raised an error: {message}"),
            Self::NotOneContext { found } => {
                write!(f, "the script returned {found} rather than one context")
            }
            Self::InstructionLimit => f.write_str("the script exceeded its instruction limit"),
            Self::MemoryLimit => f.write_str("the script exceeded its memory limit"),
            Self::OutputRefused(refusal) => write!(f, "the run refused the output: {refusal}"),
        }
    }
}

impl std::error::Error for ScriptFailure {}

/// The globals every state has that a script is not given: those that read a
/// file or compile text (`EVD_LUA_DEFAULT_STATE_EXPOSES`), those that catch an
/// error (`EVD_LUA_PCALL_SWALLOWS_LIMITS`), `collectgarbage`, which controls the
/// state's collector, and `print`, which writes to the host process's output.
/// `load` is the one that matters most - it builds a function
/// from text a script assembled, and anything removed would come back through
/// it.
const LEFT_OUT: [&str; 8] = [
    "dofile",
    "loadfile",
    "load",
    "require",
    "pcall",
    "xpcall",
    "collectgarbage",
    "print",
];

/// A Lua state holding the environment a script runs in, and nothing else.
///
/// Built from the libraries named here rather than by taking something out of
/// a default (`DEC_ENVIRONMENT_BY_NAME`): `io`, `os`, `debug`, `package` and
/// `coroutine` are never loaded, so what nobody thought of is out by
/// construction. From what is loaded, the base library's file readers, compiler
/// and error catchers go, and so does `math`'s random source, which differed
/// between processes (`EVD_LUA_RANDOM_PER_PROCESS`).
// @An environment built from named parts,IMPL_HOST_SANDBOX,impl,[CREQ_HOST_NOTHING_OUTSIDE, CREQ_HOST_NO_CATCHING]
pub(crate) fn sandbox() -> LuaResult<Lua> {
    let lua = Lua::new_with(
        LuaStdLib::STRING | LuaStdLib::TABLE | LuaStdLib::MATH | LuaStdLib::UTF8,
        LuaOptions::default(),
    )?;
    let globals = lua.globals();
    for name in LEFT_OUT {
        globals.raw_set(name, LuaNil)?;
    }
    let math: LuaTable = globals.raw_get("math")?;
    math.raw_set("random", LuaNil)?;
    math.raw_set("randomseed", LuaNil)?;
    Ok(lua)
}

/// The name a script is compiled under: its document's, marked as a file name so
/// that Lua quotes it as written in every message rather than as a string.
pub(crate) fn chunk_name(document: &str) -> String {
    format!("@{document}")
}

/// A context as a script holds it: something to call methods on and nothing
/// else. Its metatable is closed to the script (`EVD_LUA_USERDATA_PROTECTED`), so
/// what it says cannot be changed from inside.
struct Handed(Context);

impl LuaUserData for Handed {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("render", |_, this, ()| Ok(this.0.render().into_owned()));
        methods.add_method("type", |_, this, ()| {
            Ok(this.0.declared_type().as_str().to_owned())
        });
        methods.add_method("parts", |_, this, ()| {
            Ok(this
                .0
                .parts()
                .iter()
                .cloned()
                .map(Handed)
                .collect::<Vec<_>>())
        });
    }
}

/// Perform `activation` by running `script`, or say how it failed.
///
/// A new state for every call (`DEC_STATE_PER_ACTIVATION`): one shared across a
/// run was measured carrying a global and a write to the string library from
/// one node's script into the next's (`EVD_LUA_SHARED_STATE_LEAKS`).
///
/// The script is the body of the behaviour (`DEC_SCRIPT_IS_THE_BODY`) and is
/// given two arguments, read as `local given, host = ...`: the activation's
/// inputs under their parameters' names, with an unbound optional parameter
/// absent rather than empty; and the host functions, which are the context API
/// and nothing else (`DEC_HOST_FUNCTIONS_CONTEXT_API`) - `host.text(type, text)`,
/// `host.compose(type, parts, separator)`, and `host.output`, the type the
/// output is declared as.
///
/// Contexts the script makes are issued identifiers from `source`, which must be
/// the source the run's arguments came from: a second source repeats the first
/// one's identifiers, and the run refuses the collision.
// @A script run in a state of its own,IMPL_HOST_PERFORM,impl,[CREQ_HOST_RUNS_THE_SCRIPT, CREQ_HOST_FRESH_STATE]
pub(crate) fn perform(
    script: &Script,
    activation: &Activation,
    source: &mut IdSource,
    limits: Limits,
) -> Result<Context, ScriptFailure> {
    let lua = sandbox().map_err(raised)?;
    let over_instructions = limit(&lua, limits).map_err(raised)?;
    let source = RefCell::new(source);

    let returned = lua.scope(|scope| {
        let host = lua.create_table()?;
        host.raw_set("output", activation.output().as_str())?;
        host.raw_set(
            "text",
            scope.create_function(|_, (declared, text): (String, String)| {
                let declared = ContextType::new(&declared).map_err(LuaError::external)?;
                Context::text(&mut source.borrow_mut(), declared, text)
                    .map(Handed)
                    .map_err(LuaError::external)
            })?,
        )?;
        host.raw_set(
            "compose",
            scope.create_function(
                |_,
                 (declared, parts, separator): (
                    String,
                    Vec<LuaUserDataRef<Handed>>,
                    Option<String>,
                )| {
                    let declared = ContextType::new(&declared).map_err(LuaError::external)?;
                    let parts: Vec<Context> = parts.iter().map(|part| part.0.clone()).collect();
                    let separator = separator.unwrap_or_default();
                    Context::compose(&mut source.borrow_mut(), declared, &parts, &separator)
                        .map(Handed)
                        .map_err(LuaError::external)
                },
            )?,
        )?;

        let given = lua.create_table()?;
        for (parameter, context) in activation.inputs() {
            given.raw_set(parameter.as_str(), Handed(context.clone()))?;
        }

        let values: LuaMultiValue = lua
            .load(&script.source)
            .set_name(chunk_name(&script.document))
            .call((given, host))?;
        Ok(one_context(values))
    });

    // The flag is asked before the error is: the instruction limit reaches the
    // script as an ordinary error, and its message is not what says which
    // limit it was.
    if over_instructions.get() {
        return Err(ScriptFailure::InstructionLimit);
    }
    returned.map_err(failure)?
}

/// Hold `lua` to `limits`, returning the flag the instruction count sets once it
/// passes its limit.
///
/// The instruction limit is a hook every thousand instructions, measured
/// stopping an endless loop (`EVD_LUA_LIMITS_STOP`); the memory limit is the
/// state's own. Both reach the script as errors it cannot catch, because nothing
/// that catches one is in its environment (`CREQ_HOST_NO_CATCHING`).
// @Both limits set on every state,IMPL_HOST_LIMITS,impl,[CREQ_HOST_INSTRUCTION_LIMIT, CREQ_HOST_MEMORY_LIMIT]
fn limit(lua: &Lua, limits: Limits) -> LuaResult<Rc<Cell<bool>>> {
    const EVERY: u32 = 1000;
    let over = Rc::new(Cell::new(false));
    let flag = over.clone();
    let spent = Cell::new(0u64);
    lua.set_hook(
        LuaHookTriggers::new().every_nth_instruction(EVERY),
        move |_, _| {
            spent.set(spent.get() + u64::from(EVERY));
            if spent.get() > limits.instructions {
                flag.set(true);
                Err(LuaError::runtime("instruction limit exceeded"))
            } else {
                Ok(LuaVmState::Continue)
            }
        },
    )?;
    lua.set_memory_limit(limits.memory)?;
    Ok(over)
}

/// The one context `values` holds, or what they held instead.
///
/// Anything else fails, and says what it was: `nil`, a table and two contexts
/// are three different mistakes. A string is not turned into a context, since
/// the type it would be given is a decision the script did not make.
// @Exactly one context or a failure naming what came back,IMPL_HOST_ONE_CONTEXT,impl,[CREQ_HOST_ONE_CONTEXT]
fn one_context(values: LuaMultiValue) -> Result<Context, ScriptFailure> {
    let not_one = |found: String| Err(ScriptFailure::NotOneContext { found });
    let values: Vec<LuaValue> = values.into_iter().collect();
    match values.as_slice() {
        [] => not_one("nothing".to_owned()),
        [LuaValue::UserData(handed)] => match handed.borrow::<Handed>() {
            Ok(handed) => Ok(handed.0.clone()),
            Err(_) => not_one("userdata".to_owned()),
        },
        [one] => not_one(one.type_name().to_owned()),
        several => not_one(format!("{} values", several.len())),
    }
}

/// The failure a Lua error stands for, once the instruction flag has been asked.
///
/// A memory error is the memory limit, whatever raised it. Anything else is the
/// script's own error, kept whole: the document, the line and the traceback are
/// what its author reads.
// @A limit is a limit and an error is an error,IMPL_HOST_FAILURE,impl,[CREQ_HOST_ERROR_CARRIED, CREQ_HOST_MEMORY_LIMIT]
fn failure(error: LuaError) -> ScriptFailure {
    match error {
        LuaError::MemoryError(_) => ScriptFailure::MemoryLimit,
        other => raised(other),
    }
}

/// A Lua error reported as the script's own.
fn raised(error: LuaError) -> ScriptFailure {
    ScriptFailure::Raised {
        message: error.to_string(),
    }
}

#[cfg(test)]
use crate::Behaviours;
#[cfg(test)]
use crate::scripted::{
    CHAIN, CHAIN_TYPES, SMALL, appending, failed, note, rendered, run_with, workflow,
};

/// Two instances ready from the start: `a`, which runs first and whose type is
/// `first`, and `b`, designated, whose type is `second`. A script failing in `a`
/// ends the run before `b` is performed.
#[cfg(test)]
const PAIR_TYPES: &str = r#"
[types.first]
output = "note"

[types.second]
output = "note"

[types.pass]
required = { input = "note" }
output = "note"
"#;

#[cfg(test)]
const PAIR: &str = r#"
name = "pair"
output = "b"

[instances.a]
node_type = "first"

[instances.b]
node_type = "second"
"#;

/// Run the pair with `script` as `a`'s behaviour, and return how `a` failed.
///
/// That the failure is `a`'s is the evidence nothing after it was performed:
/// `b` was ready the whole time, and a run that went on would have ended on it.
#[cfg(test)]
fn first_fails_with(script: &str) -> ScriptFailure {
    let behaviours = Behaviours::new()
        .define("first", "first.lua", script)
        .define(
            "second",
            "second.lua",
            "local given, host = ...\nreturn host.text(host.output, 'b')",
        );
    let (instance, failure) = failed(run_with(&workflow(PAIR_TYPES, PAIR), &behaviours, None));
    assert_eq!(instance, "a", "the run ended on the failing instance");
    failure
}

/// Run `script` as the behaviour of an instance whose output the designated
/// instance passes on, and return what the run rendered.
#[cfg(test)]
fn first_renders(script: &str) -> String {
    let flow = r#"
name = "pair"
output = "b"

[instances.a]
node_type = "first"

[instances.b]
node_type = "pass"
bindings = { input = "a" }
"#;
    let behaviours = Behaviours::new()
        .define("first", "first.lua", script)
        .define(
            "pass",
            "pass.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.input}, '')",
        );
    rendered(run_with(&workflow(PAIR_TYPES, flow), &behaviours, None))
}

#[cfg(test)]
#[test]
fn inputs_by_parameter_name() {
    // Declared `second` then `first`: neither alphabetical nor the order the
    // script names them in.
    let types = r#"
[types.make]
required = { input = "note" }
output = "note"

[types.join]
required = { second = "note", first = "note" }
output = "note"
"#;
    let flow = r#"
name = "join"
output = "j"

[instances.one]
node_type = "make"
entry = true

[instances.two]
node_type = "make"
entry = true

[instances.j]
node_type = "join"
bindings = { first = "one", second = "two" }
"#;
    let behaviours = Behaviours::new()
        .define(
            "make",
            "make.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.input}, '')",
        )
        .define(
            "join",
            "join.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.first, given.second}, '+')",
        );

    let mut source = IdSource::new();
    let arguments = agconflo_core::Arguments::new()
        .supply("one", "input", note(&mut source, "note", "ONE"))
        .supply("two", "input", note(&mut source, "note", "TWO"));
    let ending = crate::run_scripted(
        &workflow(types, flow),
        &behaviours,
        arguments,
        &mut source,
        10,
        SMALL,
    );
    assert_eq!(rendered(ending), "ONE+TWO");
}

#[cfg(test)]
#[test]
fn unbound_optional_is_absent() {
    let types = r#"
[types.hinted]
optional = { hint = "note" }
output = "note"
"#;
    let flow = r#"
name = "hinted"
output = "h"

[instances.h]
node_type = "hinted"
"#;
    let behaviours = Behaviours::new().define(
        "hinted",
        "hinted.lua",
        "local given, host = ...\nreturn host.text(host.output, given.hint == nil and 'absent' or 'present')",
    );
    assert_eq!(
        rendered(run_with(&workflow(types, flow), &behaviours, None)),
        "absent"
    );
}

#[cfg(test)]
#[test]
fn each_type_runs_its_own_script() {
    let types = format!(
        "{CHAIN_TYPES}\n[types.other]\nrequired = {{ input = \"note\" }}\noutput = \"note\"\n"
    );
    let flow = CHAIN.replace(
        "[instances.third]\nnode_type = \"step\"",
        "[instances.third]\nnode_type = \"other\"",
    );
    assert_ne!(flow, CHAIN, "the third instance's type was changed");
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", &appending("x"))
        .define("step", "step.lua", &appending("y"))
        .define("other", "other.lua", &appending("z"));

    assert_eq!(
        rendered(run_with(
            &workflow(&types, &flow),
            &behaviours,
            Some(("first", "hello"))
        )),
        "hello x y z"
    );
}

#[cfg(test)]
#[test]
fn output_type_is_given() {
    let types = r#"
[types.summarise]
output = "summary"

[types.judge]
required = { input = "summary" }
output = "verdict"
"#;
    let flow = r#"
name = "typed"
output = "j"

[instances.s]
node_type = "summarise"

[instances.j]
node_type = "judge"
bindings = { input = "s" }
"#;
    // The same script for both, writing no type of its own.
    let script = "local given, host = ...\nreturn host.text(host.output, host.output)";
    let behaviours = Behaviours::new()
        .define("summarise", "s.lua", script)
        .define("judge", "j.lua", script);
    assert_eq!(
        rendered(run_with(&workflow(types, flow), &behaviours, None)),
        "verdict"
    );
}

#[cfg(test)]
#[test]
fn not_one_context_fails() {
    let cases = [
        ("return nil", "nil"),
        ("return", "nothing"),
        (
            "local given, host = ...\nreturn host.text(host.output, 'x'), host.text(host.output, 'y')",
            "2 values",
        ),
        ("return 'plain'", "string"),
        ("return {}", "table"),
    ];
    for (script, found) in cases {
        assert_eq!(
            first_fails_with(script),
            ScriptFailure::NotOneContext {
                found: found.to_owned()
            },
            "{script}"
        );
    }
}

#[cfg(test)]
#[test]
fn refused_output_fails_with_the_refusal() {
    // A type the node type does not declare.
    assert_eq!(
        first_fails_with("local given, host = ...\nreturn host.text('banana', 'x')"),
        ScriptFailure::OutputRefused(OutputRefusal::UndeclaredType {
            instance: "a".to_owned(),
            declared: ContextType::new("note").expect("a name"),
            reported: ContextType::new("banana").expect("a name"),
        })
    );

    // The input handed back: the run holds its identifier.
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", "local given = ...\nreturn given.input")
        .define("step", "step.lua", &appending("stepped"));
    let (instance, failure) = failed(run_with(
        &workflow(CHAIN_TYPES, CHAIN),
        &behaviours,
        Some(("first", "hello")),
    ));
    assert_eq!(instance, "first");
    assert!(
        matches!(
            failure,
            ScriptFailure::OutputRefused(OutputRefusal::IdentifierHeld { ref instance, .. })
                if instance == "first"
        ),
        "{failure:?}"
    );
}

#[cfg(test)]
#[test]
fn nothing_survives_an_activation() {
    // `second` and `third` are two instances of one type, `step`, so a state kept
    // per type is caught as well as one kept per run.
    let leaky = r#"
local given, host = ...
local seen = tostring(secret) .. '/' .. tostring(string.secret)
secret = 'left'
string.secret = 'left'
return host.compose(host.output, {given.input, host.text(host.output, seen)}, ' ')
"#;
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", leaky)
        .define("step", "step.lua", leaky);
    assert_eq!(
        rendered(run_with(
            &workflow(CHAIN_TYPES, CHAIN),
            &behaviours,
            Some(("first", "hello"))
        )),
        "hello nil/nil nil/nil nil/nil"
    );
}

#[cfg(test)]
#[test]
fn nothing_reads_outside() {
    let listing = r#"
local given, host = ...
local seen = {}
for _, name in ipairs({'io', 'os', 'require', 'package', 'dofile', 'loadfile', 'load', 'loadstring', 'debug'}) do
  if _G[name] ~= nil then seen[#seen + 1] = name end
end
if math.random ~= nil then seen[#seen + 1] = 'math.random' end
if math.randomseed ~= nil then seen[#seen + 1] = 'math.randomseed' end
return host.text(host.output, 'reachable:' .. table.concat(seen, ','))
"#;
    assert_eq!(first_renders(listing), "reachable:");

    // And reaching for one fails as the script's error, having opened nothing.
    match first_fails_with("local f = io.open('anything')") {
        ScriptFailure::Raised { message } => assert!(message.contains("'io'"), "{message}"),
        other => panic!("expected a script error, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn nothing_catches_an_error() {
    assert_eq!(
        first_renders(
            "local given, host = ...\nreturn host.text(host.output, tostring(pcall) .. tostring(xpcall) .. tostring(coroutine))"
        ),
        "nilnilnil"
    );

    // Each wraps its trouble in whatever catcher it can find, and returns
    // normally if one caught it. With none to find, the limit is what ends it.
    let guarded = |trouble: &str| {
        format!(
            r#"
local given, host = ...
local function trouble() {trouble} end
if pcall then pcall(trouble) return host.text(host.output, 'survived') end
if xpcall then xpcall(trouble, function(e) return e end) return host.text(host.output, 'survived') end
if coroutine then coroutine.resume(coroutine.create(trouble)) return host.text(host.output, 'survived') end
trouble()
"#
        )
    };
    assert_eq!(
        first_fails_with(&guarded("while true do end")),
        ScriptFailure::InstructionLimit
    );
    assert_eq!(
        first_fails_with(&guarded("return string.rep('x', 1 << 30)")),
        ScriptFailure::MemoryLimit
    );
}

#[cfg(test)]
#[test]
fn error_carries_its_message() {
    let script = "local given, host = ...\nlocal unused = 1\nerror('first line\\nsecond line')";
    match first_fails_with(script) {
        ScriptFailure::Raised { message } => {
            assert!(
                message.contains("first.lua:3:"),
                "document and line: {message}"
            );
            assert!(
                message.contains("first line\nsecond line"),
                "both lines: {message}"
            );
        }
        other => panic!("a script's error is not a limit: {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn non_string_error_is_raised() {
    for script in ["error({code = 7})", "error(nil)"] {
        assert!(
            matches!(first_fails_with(script), ScriptFailure::Raised { .. }),
            "{script}"
        );
    }
}

#[cfg(test)]
#[test]
fn runaway_script_is_stopped() {
    assert_eq!(
        first_fails_with("while true do end"),
        ScriptFailure::InstructionLimit
    );
}

/// A script spending `iterations` of an empty loop, then passing its input on.
#[cfg(test)]
fn spending(iterations: u64) -> String {
    format!(
        "local given, host = ...\nfor i = 1, {iterations} do end\nreturn host.compose(host.output, {{given.input}}, '')"
    )
}

#[cfg(test)]
#[test]
fn instruction_limit_is_per_activation() {
    // Calibrated rather than assumed: `spend` iterations fit one activation's
    // limit and twice as many do not, so the three activations of the second
    // run spend more than any limit counted across the run could allow.
    let spend = SMALL.instructions * 2 / 3;
    let chained = |seed: &str| {
        let behaviours = Behaviours::new().define("seed", "seed.lua", seed).define(
            "step",
            "step.lua",
            &spending(spend),
        );
        run_with(
            &workflow(CHAIN_TYPES, CHAIN),
            &behaviours,
            Some(("first", "hello")),
        )
    };
    let (instance, failure) = failed(chained(&spending(spend * 2)));
    assert_eq!(
        (instance.as_str(), failure),
        ("first", ScriptFailure::InstructionLimit)
    );

    assert_eq!(rendered(chained(&spending(spend))), "hello");
}

#[cfg(test)]
#[test]
fn memory_bomb_is_stopped() {
    assert_eq!(
        first_fails_with("return string.rep('x', 1 << 30)"),
        ScriptFailure::MemoryLimit
    );
}

/// A script holding a string of `bytes` while it makes its output.
#[cfg(test)]
fn holding(bytes: usize) -> String {
    format!(
        "local given, host = ...\nlocal held = string.rep('x', {bytes})\nreturn host.compose(host.output, {{given.input}}, '')"
    )
}

#[cfg(test)]
#[test]
fn memory_limit_is_per_activation() {
    // Calibrated as the instruction case is: `hold` fits one activation and
    // twice as much does not, so three activations exceed any shared limit.
    let hold = SMALL.memory * 2 / 3;
    let chained = |seed: &str| {
        let behaviours = Behaviours::new().define("seed", "seed.lua", seed).define(
            "step",
            "step.lua",
            &holding(hold),
        );
        run_with(
            &workflow(CHAIN_TYPES, CHAIN),
            &behaviours,
            Some(("first", "hello")),
        )
    };
    let (instance, failure) = failed(chained(&holding(hold * 2)));
    assert_eq!(
        (instance.as_str(), failure),
        ("first", ScriptFailure::MemoryLimit)
    );

    assert_eq!(rendered(chained(&holding(hold))), "hello");
}
