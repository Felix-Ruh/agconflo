"""Write the documents of the workflow that records approved stakeholder goals.

The documents are written next to this file. Two values a machine decides are
left as placeholders: the ubc image in manifest.toml, and the images and the
work folder in grants.template.toml. prepare.sh fills them into a run folder.
"""
import json, os

OUT = os.path.dirname(os.path.abspath(__file__))
UBC_IMAGE = "@UBC_IMAGE@"

types = {}      # name -> dict(req, desc, routes, standing)
instances = []  # (name, node_type, bindings, calls, kind, stage)
scripts = {}
roles = set()
arguments = []  # (instance, parameter, text): what a run is started with

TOOLS = {
    "run_command": ({"command": "note"}, "Run a shell command with sh in /work: busybox tools and git. The repository is at repo/."),
    "run_ubc": ({"command": "note"}, "Run a shell command with sh in /work, in a container that has ubc, the requirements tool, on its PATH. The repository is the folder repo, so its requirements project is repo/docs."),
    "read_file": ({"path": "note"}, "Read one file. path is relative to /work: the repository is the folder repo, so a path looks like repo/AGENTS.md."),
    "write_file": ({"path": "note", "text": "note"}, "Replace one file with text, entirely: text holds the whole new file. path is relative to /work, e.g. repo/docs/stakeholder/execution.rst."),
}
PARAM_HELP = {"path": "the file's path, relative to /work, e.g. repo/AGENTS.md", "text": "the whole new content of the file", "command": "the shell command to run"}


def lua_str(s):
    return json.dumps(s)


def tool_preamble(calls):
    if not calls:
        return ("You have no tools in this step. Do not call a tool and do not write out a tool call "
                "in any syntax: answer in plain text only, in exactly the form asked for below.\n\n")
    lines = ["You may call only the tools listed here. No other tool exists in this step: there is no shell, "
             "no bash and no command runner, and a call to any other name fails the step."]
    for c in calls:
        params, desc = TOOLS[c]
        lines.append("- %s(%s): %s" % (c, ", ".join(params), desc) + "".join(
            " Parameter %s: %s." % (p, PARAM_HELP[p]) for p in params))
    return "\n".join(lines) + "\n\n"


def model(name, stage, instruction, inputs, calls=(), role="drafting", selfcheck=None):
    """A model node: the tool preamble, its instruction, then each input labelled.

    With `selfcheck`, a Lua fragment reading `out` and calling `need`, the step
    checks the answer and asks again with the reasons, at most three times."""
    t = "m_" + name
    roles.add(role)
    inputs = dict(inputs)
    types[t] = dict(req=list(inputs), desc="Step: " + instruction.split(".")[0] + ".")
    parts = [lua_str(tool_preamble(calls) + instruction)] + [
        '"\\n\\n## %s\\n\\n" .. given.%s:render()' % (p, p) for p in inputs]
    if selfcheck is None:
        scripts[t] = (
            "local given, host = ...\nlocal prompt = host.text(host.output, table.concat({\n  "
            + ",\n  ".join(parts)
            + "\n}))\nreturn host.text(host.output, host.complete('%s', prompt):render())\n" % role)
    else:
        # The answer checked here, and asked for again with the reasons, at most three times.
        scripts[t] = (
            "local given, host = ...\n" + HELP + "local base = table.concat({\n  " + ",\n  ".join(parts) + "\n})\n"
            + "local out, why = nil, nil\nfor attempt = 1, 3 do\n"
            + "  local text = base\n"
            + "  if out then text = text .. \"\\n\\n## your previous answer\\n\\n\" .. out .. \"\\n\\n## why it was refused\\n\\n\" .. why .. \"\\n\\nWrite the whole answer again, correcting every point.\" end\n"
            + "  out = host.complete('%s', host.text(host.output, text)):render()\n" % role
            + "  fails = {}\n" + selfcheck + "\n  if #fails == 0 then break end\n  why = table.concat(fails, \"\\n\")\nend\n"
            + "return host.text(host.output, out)\n")
    instances.append((name, t, inputs, list(calls), "model", stage))


def script(name, stage, lua_body, inputs, kind="check", standing=False):
    t = "s_" + name
    types[t] = dict(req=list(inputs), standing=standing)
    scripts[t] = "local given, host = ...\n" + lua_body
    instances.append((name, t, dict(inputs), [], kind, stage))


def tool(name, stage, bindings, node_type="run_command"):
    instances.append((name, node_type, dict(bindings), [], "tool", stage))


def person(name, stage, node_type, inputs):
    instances.append((name, node_type, dict(inputs), [], "person", stage))


HELP = r'''
local function lines(s) local t = {} for l in (s .. "\n"):gmatch("(.-)\n") do t[#t+1] = l end return t end
local function trim(s) return (s:gsub("^%s+", ""):gsub("%s+$", "")) end
local fails = {}
local function need(ok, what) if not ok then fails[#fails+1] = what end end
local function clean(s, what)
  need(not (s:find("DSML", 1, true) or s:find("<invoke", 1, true) or s:find("<\239\189\156", 1, true)),
       what .. " is tool-call markup, not the plain text asked for")
  need(trim(s) ~= "", what .. " is empty")
end
local function suggest(id, goals)
  local words, best, bid, btitle = {}, 0, nil, nil
  for w in id:lower():gmatch("[a-z]+") do if #w > 2 and w ~= "stkh" then words[#words+1] = w end end
  for gid, title in (goals or ""):gmatch("\n(STKH_[A-Z0-9_]+)%s+(.-)%s%s+") do
    local score, t = 0, title:lower()
    for _, w in ipairs(words) do if t:find(w, 1, true) then score = score + 1 end end
    if score > best then best, bid, btitle = score, gid, title end
  end
  if bid then return " (did you mean " .. bid .. ', "' .. btitle .. '"?)' end
  return ""
end
local function report(name)
  if #fails == 0 then return host.text(host.output, "PASS " .. name) end
  return host.text(host.output, "FAIL " .. name .. ": " .. table.concat(fails, "; "))
end
'''


def command(name, stage, lua, inputs, tool_type="run_command"):
    """A script writing a command, and the wired tool step running it."""
    script("cmd_" + name, stage, lua, inputs, kind="command")
    tool(name, stage, {"command": "cmd_" + name}, tool_type)



# ================================================================================================
# One workflow for the task "record approved stakeholder goals", for any number of goals in
# any stakeholder files. Stage A reads the repository once; stage B is a loop over the goals, one
# attempt per pass, the router `judge` holding the state; stage C checks the whole change.
# Accepted goals are staged in git's index; an attempt starts from the index.
# ================================================================================================

GOAL_NOTE = (" The goal below says which goal of how many this is and which pass. If its FEEDBACK is not "
             "'none', a reviewer found what it says wrong with the last attempt at this goal: correct "
             "whatever of it concerns your answer.")

# ---- A: once ----------------------------------------------------------------------------------
script("brief", "read", "return host.text(host.output, given.brief:render())\n", {"brief": None}, kind="entry")
script("statements", "read", HELP + r'''
local out, seen = {}, {}
for s in given.brief:render():gmatch('"(Agconflo shall[^"]*)"') do
  if not seen[s] then seen[s] = true; out[#out+1] = s end
end
return host.text(host.output, table.concat(out, "\n"))
''', {"brief": "brief"}, kind="split")
script("given_stakeholders", "read", HELP + r"""
local out, seen = {}, {}
for _, l in ipairs(lines(given.brief:render())) do
  local s = l:match('"(Agconflo shall[^"]*)"')
  if s and not seen[s] then
    seen[s] = true
    out[#out+1] = l:match("%(stakeholder:%s*(%a+)%)") or "unset"
  end
end
return host.text(host.output, table.concat(out, "\n"))
""", {"brief": "brief"}, kind="split")
script("v_statements", "read", HELP + r'''
local n = 0
for _, l in ipairs(lines(given.statements:render())) do if trim(l) ~= "" then n = n + 1 end end
need(n >= 1, "the brief quotes no statement beginning 'Agconflo shall'")
return report("statements copied from the brief (" .. n .. ")")
''', {"statements": "statements"})
command("listing", "read", 'return host.text(host.output, [[for f in repo/docs/stakeholder/*.rst; do echo "== $f: $(grep -c "^.. stkh_req::" "$f") goals"; sed -n "1,14p" "$f"; grep -A1 "^.. stkh_req::" "$f" | grep -v "^--"; done]])\n', {"brief": "brief"})
command("goals", "read", 'return host.text(host.output, [[cd repo && ubc query cypher --project docs --strict "MATCH (n:stkh_req) RETURN n.id, n.title, n.statement, n.stakeholder ORDER BY n.id"]])\n', {"brief": "brief"}, tool_type="run_ubc")
command("named", "read", HELP + r'''
local ids, seen = {}, {}
for id in given.brief:render():gmatch("[A-Z][A-Z0-9]*_[A-Z0-9_]+") do
  if not seen[id] then seen[id] = true; ids[#ids+1] = '"' .. id .. '"' end
end
if #ids == 0 then ids[1] = '"NONE"' end
return host.text(host.output, "cd repo && ubc query cypher --project docs --strict 'MATCH (n) WHERE n.id IN [" .. table.concat(ids, ", ") .. "] RETURN n.id, n.title, coalesce(n.statement, n.observation) AS says ORDER BY n.id'")
''', {"brief": "brief"}, tool_type="run_ubc")
# Every section of AGENTS.md and README.md whose heading mentions requirements, to its next heading
# of the same or a higher level.
command("rules", "read", r'''return host.text(host.output, [[for f in repo/AGENTS.md repo/README.md; do awk -v F="$f" '/^```/{fence=!fence} !fence && /^#+ /{lvl=length($1); if(p && lvl<=plvl) p=0; if(!p && tolower($0) ~ /requirement/){p=1; plvl=lvl; print "== " F}} p' "$f"; done]])
''', {"brief": "brief"})
command("stakeholders", "read", r'''return host.text(host.output, [[awk '/^\[needs.fields.stakeholder\]/{p=1} p&&/^schema/{print; exit} p' repo/docs/ubproject.toml]])
''', {"brief": "brief"})
command("example", "read", r'''return host.text(host.output, [[f=$(grep -c "^.. stkh_req::" repo/docs/stakeholder/*.rst | sort -t: -k2 -n | tail -1 | cut -d: -f1); awk '/^\.\. stkh_req::/{buf=""; p=1} p{buf=buf $0 "\n"} END{printf "%s", buf}' "$f"]])
''', {"brief": "brief"})
script("v_read", "read", HELP + r'''
need(#given.rules:render() > 400 and given.rules:render():lower():find("requirement", 1, true) ~= nil, "no section about requirements came back from AGENTS.md or README.md")
need(given.example:render():find(":statement:", 1, true) ~= nil, "the example goal has no statement")
need(given.goals:render():find("STKH_", 1, true) ~= nil, "no stakeholder goal came back from the graph")
need(given.stakeholders:render():find("enum", 1, true) ~= nil, "the stakeholder field's definition was not found")
return report("the repository was read")
''', {"rules": "rules", "example": "example", "goals": "goals", "stakeholders": "stakeholders"})

# ---- B: the loop over the goals ------------------------------------------------------------------
# head: the attempt this pass makes, from the judge's state.
script("head", "loop", HELP + r'''
local st = "\n" .. given.state:render()
local k = tonumber(st:match("\ngoal (%d+)\n") or "1")
local p = tonumber(st:match("\npass (%d+)\n") or "1")
local stage = st:match("\nstage (%a+)\n") or "no"
local fb = st:match("feedback:\n(.*)$") or ""
if trim(fb) == "" then fb = "none" end
local ss = {}
for _, l in ipairs(lines(given.statements:render())) do if trim(l) ~= "" then ss[#ss+1] = trim(l) end end
local ws = lines(given.given_stakeholders:render())
return host.text(host.output, "GOAL " .. k .. " OF " .. #ss .. "\nPASS " .. p .. "\nSTAGE " .. stage ..
  "\nSTATEMENT: " .. (ss[k] or "") .. "\nSTAKEHOLDER GIVEN: " .. trim(ws[k] or "unset") .. "\nFEEDBACK:\n" .. fb)
''', {"state": "judge", "statements": "statements", "given_stakeholders": "given_stakeholders"}, kind="head")
arguments.append(("head", "state", "goal 1\npass 1\nstage no\nfeedback:\n"))

GOAL_OF = r'''local g = given.goal:render()
local statement = g:match("STATEMENT: ([^\n]*)") or ""
'''

command("prep", "loop", GOAL_OF + r'''
local first = g:match("^GOAL 1 OF") and g:match("\nPASS 1\n")
local cmd
if first then cmd = "git -C repo reset -q --hard && git -C repo clean -fdq"
elseif g:match("\nSTAGE yes\n") then cmd = "git -C repo add -A && git -C repo checkout -q -- . && git -C repo clean -fdq"
else cmd = "git -C repo checkout -q -- . && git -C repo clean -fdq" end
return host.text(host.output, cmd .. " && git -C repo status --porcelain && echo ready")
''', {"goal": "head"})

model("pick_file", "loop",
      "Choose the one file under repo/docs/stakeholder/ the goal below belongs in, from the listing of those files with their introductions and their goals. Answer with a first line 'FILE: ' followed by the path exactly as listed, then one sentence saying why, naming goals already in that file." + GOAL_NOTE,
      {"goal": "head", "listing": "listing"},
      selfcheck=r'''
  clean(out, "the answer")
  local f = out:match("FILE:%s*(%S+)")
  need(f ~= nil, "there is no line starting 'FILE: '")
  if f then need(given.listing:render():find("== " .. f .. ":", 1, true) ~= nil, f .. " is not a file in the listing") end
''')
FILE_OF = r'local file = given.file:render():match("FILE:%s*(%S+)") or "repo/NONE"' + "\n"
command("current", "loop", FILE_OF + 'return host.text(host.output, "cat " .. file)\n', {"file": "pick_file", "prep": "prep"})
script("file_goals", "loop", HELP + r'''
local out, title, id = {}, nil, nil
for _, l in ipairs(lines(given.current:render())) do
  local t = l:match("^%.%. stkh_req:: (.*)$")
  if t then title = t end
  local i = l:match("^   :id: (STKH_[A-Z0-9_]+)")
  if i then id = i end
  local who = l:match("^   :stakeholder: (%a+)")
  if who and id and title then out[#out+1] = id .. " - " .. title .. " (stakeholder: " .. who .. ")"; title, id = nil, nil end
end
return host.text(host.output, table.concat(out, "\n"))
''', {"current": "current"}, kind="split")

model("keywords", "loop",
      "Choose words to search the repository's decisions for any that bear on the goal below. Answer with 4 to 8 lowercase words, one per line, each a single word of letters only, nothing else. Choose words specific to what the goal asks for, which a decision about it would use." + GOAL_NOTE,
      {"goal": "head"},
      selfcheck=r'''
  clean(out, "the answer")
  local n = 0
  for _, l in ipairs(lines(trim(out))) do
    n = n + 1
    need(trim(l):match("^[a-z]+$") ~= nil, "not one lowercase word: " .. trim(l))
  end
  need(n >= 4 and n <= 8, "expected 4 to 8 words, got " .. n)
''')
command("decs", "loop", HELP + r'''
local ks = {}
for _, l in ipairs(lines(given.keywords:render())) do
  local k = trim(l):lower()
  if k:match("^[a-z]+$") then ks[#ks+1] = '"' .. k .. '"' end
end
if #ks == 0 then ks[1] = '"nothing"' end
return host.text(host.output, "cd repo && ubc query cypher --project docs --strict 'MATCH (n:dec) WITH n, size([k IN [" .. table.concat(ks, ", ") .. "] WHERE toLower(n.title) CONTAINS k OR toLower(n.statement) CONTAINS k OR toLower(n.content) CONTAINS k]) AS hits WHERE hits > 0 RETURN n.id, n.title, n.statement, hits ORDER BY hits DESC, n.id LIMIT 20'")
''', {"keywords": "keywords"}, tool_type="run_ubc")

KNOWN = 'local known = given.goals:render() .. "\\n" .. given.decs:render() .. "\\n" .. given.named:render() .. "\\n" .. given.current:render()\n'
model("neighbours", "loop",
      "From the needs below, list the existing needs the goal's body should name: any the brief names that concern this goal, those it depends on, those it could be mistaken for, and decisions that already make room for it. Give 2 to 4 lines, one per need, first any need the brief names that concerns this goal, each as the id, a colon, and one sentence copied exactly from that need's statement as it appears below. Use only ids that appear below." + GOAL_NOTE,
      {"goal": "head", "brief": "brief", "named": "named", "goals": "goals", "decs": "decs", "current": "current"},
      selfcheck=r'''
  clean(out, "the answer")
  ''' + KNOWN + r'''
  local n = 0
  for _, l in ipairs(lines(trim(out))) do
    local id, q = l:match("^%W*([A-Z][A-Z0-9]*_[A-Z0-9_]+)%W*:%s*(.-)%s*$")
    if id then
      n = n + 1
      need(known:find(id, 1, true) ~= nil, "no such need: " .. id .. suggest(id, given.goals:render()))
      q = q:gsub('^"', ""):gsub('"$', "")
      need(#q > 10 and known:find(q:sub(1, 60), 1, true) ~= nil, "the sentence given for " .. id .. " is not copied exactly from what is listed")
    elseif trim(l) ~= "" then need(false, "a line is not 'ID: sentence': " .. l:sub(1, 60)) end
  end
  need(n >= 2 and n <= 4, "expected 2 to 4 needs, got " .. n)
''')

model("naming", "loop",
      "Name the goal below as a stakeholder requirement of this repository. Answer with exactly five lines:\nID: STKH_ followed by upper-case words joined by underscores, not used by any goal below\nTITLE: a short sentence saying what the goal gives, in the style of the titles below, without a full stop\nPRECEDENT: the id of the existing goal below most like this one in whose goal it is\nSTAKEHOLDER: the goal's STAKEHOLDER GIVEN if it is not unset; otherwise user, agent or maintainer: the stakeholder of that precedent, unless the definition below plainly says otherwise\nWHY: one sentence saying why that stakeholder, naming who would miss this goal if it were not met." + GOAL_NOTE,
      {"goal": "head", "stakeholders": "stakeholders", "goals": "goals", "file_goals": "file_goals"},
      selfcheck=r"""
  clean(out, "the answer")
  local id = out:match("ID:%s*(STKH_[A-Z0-9_]+)%s*\n")
  need(id ~= nil, "no line 'ID: STKH_...'")
  if id then need(not (given.goals:render() .. "\n" .. given.file_goals:render()):find(id .. " ", 1, true), id .. " is already a goal's id") end
  local t = out:match("TITLE:%s*([^\n]+)")
  need(t ~= nil and #t > 8 and not t:match("%.%s*$"), "no line 'TITLE: <title>' without a full stop")
  local who = out:match("STAKEHOLDER:%s*(%a+)")
  need(who == "user" or who == "agent" or who == "maintainer", "the stakeholder is not user, agent or maintainer")
  local given_who = given.goal:render():match("STAKEHOLDER GIVEN: (%a+)") or "unset"
  if given_who ~= "unset" then need(who == given_who, "the task gives the stakeholder " .. given_who .. " for this goal") end
  local pre = out:match("PRECEDENT:%s*(STKH_[A-Z0-9_]+)")
  need(pre ~= nil, "no line 'PRECEDENT: <id of an existing goal>'")
  if pre and given_who == "unset" then
    local row = ("\n" .. given.goals:render()):match("\n" .. pre .. "%s[^\n]*")
    need(row ~= nil, pre .. " is not an existing goal")
    if row and who and not row:find("%s" .. who .. "%s*$") then
      need(out:match("WHY:[^\n]*[Dd]efinition") ~= nil, "the stakeholder differs from the precedent " .. pre .. "'s, and WHY does not say which part of the definition decides it")
    end
  end
""")

model("place", "loop",
      "Say after which goal of the file the goal below goes: beside the goals it is most closely related to. Answer with exactly one line 'AFTER: ' followed by an id from the file's goals listed below, in their order, and nothing else." + GOAL_NOTE,
      {"goal": "head", "file_goals": "file_goals", "neighbours": "neighbours"},
      selfcheck=r'''
  clean(out, "the answer")
  local a = out:match("AFTER:%s*(STKH_[A-Z0-9_]+)")
  need(a ~= nil, "no line 'AFTER: <id>'")
  if a then need(("\n" .. given.file_goals:render()):find("\n" .. a .. " ", 1, true) ~= nil, a .. " is not a goal in the file") end
''')

INTRO_CHECK = r"""
  clean(out, "the introduction")
  if trim(out) ~= "UNCHANGED" then
    local n, first = 0, lines(given.introduction:render())[1] or ""
    for _, l in ipairs(lines(trim(out))) do
      n = n + 1
      need(#l <= 79, "line " .. n .. " is longer than 79 columns")
      need(not l:match("^%.%. ") and not l:match("^   :"), "line " .. n .. " is not prose")
    end
    need(n <= 10, "it is longer than ten lines")
    local s = out:gsub("``[^`]*``", "")
    need(not s:find(", [A-Z]"), "a clause after a comma starts with a capital letter: every theme is a lowercase clause like the others")
    need(trim(out):sub(1, 30) == first:sub(1, 30), "it does not begin as the introduction does: " .. first:sub(1, 30))
  end
"""
script("intro0", "loop", HELP + r"""
local ls, out, started = lines(given.current:render()), {}, false
for i = 2, #ls do
  local l = ls[i]
  if l:match("^%.%. ") then break end
  if not l:match("^=+$") and trim(l) ~= "" and i > 4 then started = true; out[#out+1] = l
  elseif started and trim(l) == "" then break end
end
return host.text(host.output, table.concat(out, "\n"))
""", {"current": "current"}, kind="split")
model("intro", "loop",
      "Below is the introduction of the file the goal goes in. It names the themes the file's goals cover, as short lowercase clauses. If the goal below adds a theme the introduction does not name yet, write the introduction again with that theme added as one more clause in the same style, changing nothing else, wrapped at 79 columns. Otherwise answer exactly UNCHANGED. Answer with the paragraph or UNCHANGED and nothing else - no heading, no quotes, no commentary." + GOAL_NOTE,
      {"goal": "head", "introduction": "intro0", "file_goals": "file_goals"},
      selfcheck=INTRO_CHECK)

model("draft", "loop",
      "Write one stkh_req directive for the goal below and nothing else: no fences, no commentary. Follow the example goal's form exactly: '.. stkh_req:: <title>', then the options indented three spaces (:id:, :stakeholder:, :statement: with the goal's statement copied exactly), a blank line, and a body indented three spaces, wrapped at 79 columns, in three paragraphs: why the goal exists; what it deliberately does not say; and how it stands beside each need in the neighbours list, naming each by its id in double backquotes and saying for each in a sentence or two how the two differ or depend - what either could hold without the other - rather than listing them. Use exactly the id, title and stakeholder the naming gives. Name no need that is not in the neighbours list. Follow the rules below." + GOAL_NOTE,
      {"goal": "head", "naming": "naming", "neighbours": "neighbours", "example": "example", "rules": "rules", "goals": "goals", "decs": "decs", "current": "current"},
      selfcheck=GOAL_OF.replace("local g = given.goal", "  local g = given.goal") + r'''
  local d = out
  clean(d, "the draft")
  local ls = lines(trim(d))
  local nm = given.naming:render()
  local want_id = nm:match("ID:%s*(STKH_[A-Z0-9_]+)") or ""
  local want_title = trim(nm:match("TITLE:%s*([^\n]+)") or "")
  local want_who = nm:match("STAKEHOLDER:%s*(%a+)") or ""
  need(ls[1] == ".. stkh_req:: " .. want_title, "the first line is not '.. stkh_req:: " .. want_title .. "'")
  need(d:find("\n   :id: " .. want_id .. "\n", 1, true) ~= nil, "the id is not ':id: " .. want_id .. "'")
  need(d:find("\n   :stakeholder: " .. want_who .. "\n", 1, true) ~= nil, "the stakeholder is not ':stakeholder: " .. want_who .. "'")
  need(d:match("\n   :statement: ([^\n]*)") == statement, "the statement is not exactly: " .. statement)
  local body = 0
  for i, l in ipairs(ls) do
    if l ~= "" and not l:match("^   ") and i > 1 then need(false, "line " .. i .. " is not indented three spaces") break end
    if #l > 80 and not l:match("^   :statement:") then need(false, "line " .. i .. " is longer than 80 columns") break end
    if l:match("^   [^:]") then body = body + 1 end
  end
  need(body >= 6, "the body is shorter than six lines")
  for id in d:gmatch("[^`]`([A-Z][A-Z0-9]*_[A-Z0-9_]+)`[^`]") do need(false, id .. " is in single backquotes: write ``" .. id .. "``") end
  local nb = given.neighbours:render()
  local seen = {}
  for id in d:gmatch("[A-Z][A-Z0-9]*_[A-Z0-9_]+") do
    if id ~= want_id and not seen[id] then
      seen[id] = true
      need(nb:find(id, 1, true) ~= nil, id .. " is not in the neighbours list" .. suggest(id, given.goals:render()))
    end
  end
  for id in nb:gmatch("([A-Z][A-Z0-9]*_[A-Z0-9_]+)%W*:") do need(seen[id], "the body does not name " .. id .. ", which the neighbours list gives") end
''')

script("splice", "loop", HELP + r'''
local ls = lines(given.current:render())
table.remove(ls, 1)
while #ls > 0 and ls[#ls] == "" do table.remove(ls) end
local a = given.place:render():match("AFTER:%s*(STKH_[A-Z0-9_]+)")
local d = lines(trim(given.draft:render()))
local at
for i, l in ipairs(ls) do if l == "   :id: " .. (a or "") then at = i end end
if at then
  local stop = #ls + 1
  for i = at + 1, #ls do if ls[i]:match("^%.%. ") then stop = i break end end
  local out = {}
  for i = 1, stop - 1 do out[#out+1] = ls[i] end
  while #out > 0 and out[#out] == "" do table.remove(out) end
  out[#out+1] = ""
  for _, l in ipairs(d) do out[#out+1] = l end
  out[#out+1] = ""
  for i = stop, #ls do out[#out+1] = ls[i] end
  ls = out
end
local intro = trim(given.intro:render())
if intro ~= "UNCHANGED" and intro ~= "" then
  local st, en
  for i = 4, #ls do
    if not st and trim(ls[i]) ~= "" and not ls[i]:match("^=+$") then st = i end
    if st and trim(ls[i]) == "" then en = i - 1 break end
  end
  if st and en and not ls[st]:match("^%.%. ") then
    local out = {}
    for i = 1, st - 1 do out[#out+1] = ls[i] end
    for _, l in ipairs(lines(intro)) do out[#out+1] = l end
    for i = en + 1, #ls do out[#out+1] = ls[i] end
    ls = out
  end
end
while #ls > 0 and ls[#ls] == "" do table.remove(ls) end
local text = (table.concat(ls, "\n") .. "\n"):gsub("\n\n\n+", "\n\n")
return host.text(host.output, text)
''', {"current": "current", "place": "place", "draft": "draft", "intro": "intro"}, kind="splice")
script("path", "loop", FILE_OF + "return host.text(host.output, file)\n", {"file": "pick_file"}, kind="split")
tool("write", "loop", {"path": "path", "text": "splice"}, "write_file")
command("ubc", "loop", 'return host.text(host.output, "cd repo/docs && ubc check --deny warning 2>&1 | tail -30")\n', {"write": "write"}, tool_type="run_ubc")
command("diff", "loop", 'return host.text(host.output, "git -C repo diff")\n', {"write": "write"})

# ---- B review: one question per step -------------------------------------------------------------
BODY_SENTENCES = r'''
local function body_sentences(d)
  local text = {}
  for _, l in ipairs(lines(d)) do if l:match("^   [^:%s]") then text[#text+1] = trim(l) end end
  local s, out = table.concat(text, " "), {}
  for sent in (s .. " "):gmatch("(.-[%.;:])%s") do if trim(sent) ~= "" then out[#out+1] = trim(sent) end end
  return out
end
'''
script("closing", "review", HELP + BODY_SENTENCES + r'''
local out, k = {}, 0
for _, sent in ipairs(body_sentences(given.draft:render())) do
  local low = " " .. sent:lower():gsub("[^%a ]", " ") .. " "
  for _, w in ipairs({ " only ", " each ", " every ", " exactly ", " no other " }) do
    if low:find(w, 1, true) then k = k + 1; out[#out+1] = "W" .. k .. ': "' .. sent .. '" uses "' .. trim(w) .. '"'; break end
  end
end
if #out == 0 then return host.text(host.output, "none") end
return host.text(host.output, table.concat(out, "\n"))
''', {"draft": "draft"}, kind="check")

model("claims", "review",
      "List every sentence of the draft's body below that states how Agconflo behaves, what it already does, or what another need says, leaving out sentences that name a need by its id (another step checks those), sentences giving the reason the goal exists and sentences saying what the goal deliberately does not say. For each, one line: 'C<n>: \"<the sentence>\" - SUPPORTED BY <ID>', naming a need listed below that says it, or 'C<n>: \"<the sentence>\" - UNSUPPORTED' when none does. A need supports a sentence only if it says the same thing, not merely the same topic. If there is no such sentence, answer exactly NONE.",
      {"draft": "draft", "named": "named", "goals": "goals", "decs": "decs", "current": "current"}, role="reviewing",
      selfcheck=r'''
  clean(out, "the answer")
  ''' + KNOWN + r'''
  if trim(out) ~= "NONE" then
    local n = 0
    for l in out:gmatch("[^\n]+") do
      if l:match("^%W*C%d+") then
        n = n + 1
        local id = l:match("SUPPORTED BY%W*([A-Z][A-Z0-9_]+)")
        need(id ~= nil or l:find("UNSUPPORTED", 1, true), "a claim without SUPPORTED BY <ID> or UNSUPPORTED: " .. l:sub(1, 60))
        if id then need(known:find(id, 1, true) ~= nil, "no such need: " .. id) end
      end
    end
    need(n >= 1, "no line 'C<n>: ...', and not NONE")
  end
''')

model("note", "review",
      "Below is the task, which may carry a maintainer's note, and a draft for one of its goals. List each sentence of the maintainer's note that concerns this goal, and say whether the draft keeps it: one line each, 'N<k>: \"<the note's sentence, copied exactly>\" - KEPT', or '- MISSING' when the draft does not say it, or '- CONTRADICTED' when the draft says the opposite, then ' - ' and the draft's words that show it. If the task has no maintainer's note, or none of its sentences concerns this goal, answer exactly NONE.",
      {"brief": "brief", "goal": "head", "draft": "draft"}, role="reviewing",
      selfcheck=r"""
  clean(out, "the answer")
  if trim(out) ~= "NONE" then
    local b, n = given.brief:render(), 0
    for l in out:gmatch("[^\n]+") do
      if l:match("^%W*N%d+") then
        n = n + 1
        local q = l:match('"(.-)"')
        need(q ~= nil and #q > 12 and b:find(q:sub(1, 50), 1, true) ~= nil, "a note sentence not copied exactly from the task: " .. l:sub(1, 60))
        need(l:find("KEPT", 1, true) or l:find("MISSING", 1, true) or l:find("CONTRADICTED", 1, true), "a line without KEPT, MISSING or CONTRADICTED: " .. l:sub(1, 60))
      end
    end
    need(n >= 1, "no line 'N<k>: ...', and not NONE")
  end
""")

script("pairs", "review", HELP + BODY_SENTENCES + r'''
local rows = given.goals:render() .. "\n" .. given.decs:render() .. "\n" .. given.named:render()
local cur = given.current:render()
local function statement_of(id)
  local s = rows:match("\n" .. id .. "%s%s+.-%s%s+([^\n]-)%s*\n")
  if s then s = s:gsub("%s+%d+$", ""):gsub("%s%s+%a+$", "") end
  if not s then s = cur:match(":id: " .. id .. "\n.-:statement: ([^\n]*)") end
  return s
end
local out, k = {}, 0
for _, sent in ipairs(body_sentences(given.draft:render())) do
  for id in sent:gmatch("``([A-Z][A-Z0-9]*_[A-Z0-9_]+)``") do
    local st = statement_of(id)
    if st then k = k + 1; out[#out+1] = "P" .. k .. ": " .. sent .. "\n    " .. id .. " says: " .. st end
  end
end
if #out == 0 then return host.text(host.output, "none") end
return host.text(host.output, table.concat(out, "\n"))
''', {"draft": "draft", "named": "named", "goals": "goals", "decs": "decs", "current": "current"}, kind="check")

model("relations", "review",
      "Each pair below is a sentence from a new requirement's body and the statement of a need it names. For each pair, say whether the sentence describes that need and its relation to the new goal correctly, given only what the need's statement says: one line each, 'P<k>: CORRECT' or 'P<k>: WRONG - <what the statement says instead>'. If the pairs are 'none', answer exactly NONE.",
      {"pairs": "pairs"}, role="reviewing",
      selfcheck=r'''
  clean(out, "the answer")
  local p = given.pairs:render()
  if trim(p) == "none" then need(trim(out) == "NONE", "there are no pairs, so the answer is NONE")
  else
    for k in p:gmatch("P(%d+):") do
      need(out:find("P" .. k .. ": CORRECT", 1, true) or out:find("P" .. k .. ": WRONG", 1, true), "P" .. k .. " is not answered CORRECT or WRONG")
    end
  end
''')

# ---- B judge: the loop's router ------------------------------------------------------------------
script("judge", "loop", HELP + r'''
local POLICY = table.concat({
  "A body states nothing about how Agconflo behaves, or about what a need says, that no need supports.",
  "A body describes each need it names as that need's statement says.",
  "A body keeps every sentence of the maintainer's note that concerns its goal, and contradicts none.",
  "Words that close the world - only, each, every, exactly, no other - are allowed where the parent closes it too, and nowhere else; a stakeholder goal's parent, its stakeholder, closes nothing, so its body declares no set of options or mechanisms complete.",
}, "\n")
local g = given.goal:render()
local k = tonumber(g:match("GOAL (%d+)")) or 1
local n = tonumber(g:match("OF (%d+)")) or 1
local p = tonumber(g:match("PASS (%d+)")) or 1
local prev = given.previous:render()
local notes = prev:match("notes:\n(.-)\nfeedback:") or ""
local ubc = given.ubc:render()
local hard = {}
if not ubc:find("No errors found.", 1, true) then hard[#hard+1] = "ubc check reported: " .. ubc:sub(1, 900) end
local findings = {}
for l in given.closing:render():gmatch("[^\n]+") do if l:match("^W%d+:") then findings[#findings+1] = "Closing word. " .. l end end
for l in given.claims:render():gmatch("[^\n]+") do if l:find("UNSUPPORTED", 1, true) and not l:find("``", 1, true) then findings[#findings+1] = "Unsupported claim. " .. l end end
for l in given.note:render():gmatch("[^\n]+") do if l:find("MISSING", 1, true) or l:find("CONTRADICTED", 1, true) then findings[#findings+1] = "Maintainer's note. " .. l end end
for l in given.relations:render():gmatch("[^\n]+") do if l:find(": WRONG", 1, true) then findings[#findings+1] = "Wrong relation. " .. l end end
local defects, how = {}, nil
if #hard > 0 then
  defects, how = hard, "ubc check failed"
elseif #findings > 0 then
  local listed = {}
  for i, f in ipairs(findings) do listed[#listed+1] = "F" .. i .. ": " .. f end
  local base = table.concat({
    "You have no tools in this step. Do not call a tool and do not write out a tool call: answer in plain text only.",
    "",
    "You judge findings a reviewer raised about a new requirement in the diff below. For each finding, fill in this form, one field per line, and nothing else:",
    "F<k>:",
    "FOUND: the finding in your own words",
    "RULE: one sentence copied exactly from the task, the rules or the review policy below that the finding depends on, or NONE",
    "DIFF: the sentence of the diff the finding is about, copied exactly",
    "CLOSES: for a closing-word finding, the set of options or mechanisms the sentence declares complete, or NONE if it declares no set complete; for any other finding, NONE",
    "DEFECT: yes if the diff breaks that rule or the task, no otherwise",
    "A finding whose RULE is NONE is not a defect, and neither is a closing-word finding whose CLOSES is NONE. After the last form, one line: VERDICT: revise if any DEFECT is yes, otherwise VERDICT: accept.",
    "", "## task", "", given.brief:render(),
    "", "## rules", "", given.rules:render(),
    "", "## review policy", "", POLICY,
    "", "## diff", "", given.diff:render(),
    "", "## findings", "", table.concat(listed, "\n"),
  }, "\n")
  local src = given.brief:render() .. "\n" .. given.rules:render() .. "\n" .. POLICY
  local answer, why = nil, nil
  for attempt = 1, 2 do
    local text = base
    if answer then text = text .. "\n\n## your previous answer\n\n" .. answer .. "\n\n## why it was refused\n\n" .. why .. "\n\nAnswer again, correcting every point." end
    answer = host.complete("judging", host.text("note", text)):render()
    local missing = {}
    for i = 1, #findings do if not answer:find("F" .. i .. ":", 1, true) then missing[#missing+1] = "F" .. i .. " has no form" end end
    if not answer:match("VERDICT:%s*%a+") then missing[#missing+1] = "there is no VERDICT line" end
    if #missing == 0 then break end
    why = table.concat(missing, "\n")
  end
  -- A defect counts only when the rule it rests on is quoted from the rules or the task.
  for i = 1, #findings do
    local form = answer:match("F" .. i .. ":(.-)\nF" .. (i + 1) .. ":") or answer:match("F" .. i .. ":(.-)VERDICT") or ""
    local rule = trim(form:match("RULE:%s*([^\n]*)") or "NONE"):gsub('^"', ""):gsub('"$', "")
    local yes = (form:match("DEFECT:%s*(%a+)") or ""):lower() == "yes"
    -- A closing word that declares no set complete is not the one the rule forbids.
    if findings[i]:match("^Closing word") and trim(form:match("CLOSES:%s*([^\n]*)") or "NONE"):upper():match("^NONE") then yes = false end
    local quoted = rule ~= "NONE" and #rule > 12 and src:find(rule:sub(1, 50), 1, true) ~= nil
    if yes and quoted then defects[#defects+1] = findings[i] .. " -> breaks: " .. rule end
  end
  how = #defects .. " of " .. #findings .. " finding(s) judged defects, each resting on a quoted rule"
else
  how = "no finding"
end
local state
if #defects > 0 and p < 3 then
  host.route({ "head", "judge" })
  state = "goal " .. k .. "\npass " .. (p + 1) .. "\nstage no"
  local out = { "revise goal " .. k .. " (pass " .. p .. "): " .. how, state, "notes:\n" .. notes, "feedback:" }
  for _, d in ipairs(defects) do out[#out+1] = "- " .. d end
  return host.text(host.output, table.concat(out, "\n"))
end
if #defects > 0 then notes = notes .. "goal " .. k .. " kept after pass " .. p .. " with open defects: " .. table.concat(defects, " | ") .. "\n" end
if k < n then
  host.route({ "head", "judge" })
  return host.text(host.output, "accept goal " .. k .. " (pass " .. p .. "): " .. how .. "\ngoal " .. (k + 1) .. "\npass 1\nstage yes\nnotes:\n" .. notes .. "\nfeedback:\n")
end
host.route({ "cmd_fin", "describe", "approval" })
return host.text(host.output, "accept goal " .. k .. " (pass " .. p .. "): " .. how .. ", the last goal\nnotes:\n" .. notes .. "\nfeedback:\n")
''', {"goal": "head", "previous": "judge", "ubc": "ubc", "closing": "closing", "claims": "claims", "relations": "relations",
      "diff": "diff", "brief": "brief", "rules": "rules", "note": "note"}, kind="router")
types["s_judge"]["routes"] = True
arguments.append(("judge", "previous", "start\nnotes:\n\nfeedback:\n"))

# ---- C: the whole change -------------------------------------------------------------------------
command("fin", "final", 'return host.text(host.output, "git -C repo add -A && git -C repo status --porcelain")\n', {"verdict": "judge"})
command("fdiff", "final", 'return host.text(host.output, "git -C repo diff --cached")\n', {"fin": "fin"})
command("fubc", "final", 'return host.text(host.output, "cd repo/docs && ubc check --deny warning 2>&1 | tail -30")\n', {"fin": "fin"}, tool_type="run_ubc")
script("v_final", "final", HELP + r'''
local d, b = given.fdiff:render(), given.brief:render()
need(given.fubc:render():find("No errors found.", 1, true) ~= nil, "ubc check of the whole change reported errors")
for _, s in ipairs(lines(given.statements:render())) do
  if trim(s) ~= "" then need(d:find("+   :statement: " .. trim(s), 1, true) ~= nil, "not recorded: " .. trim(s)) end
end
local ws = lines(given.given_stakeholders:render())
local k = 0
for _, s in ipairs(lines(given.statements:render())) do
  if trim(s) ~= "" then
    k = k + 1
    local w = trim(ws[k] or "unset")
    if w ~= "unset" then need(d:find("+   :stakeholder: " .. w .. "\n+   :statement: " .. trim(s), 1, true) ~= nil, "not recorded with the stakeholder " .. w .. ": " .. trim(s)) end
  end
end
for f in d:gmatch("\n%+%+%+ b/(%S+)") do need(f:match("^docs/stakeholder/") ~= nil, "changed outside docs/stakeholder: " .. f) end
local named, missing = 0, {}
for id in b:gmatch("[A-Z][A-Z0-9]*_[A-Z0-9_]+") do
  named = named + 1
  if not d:find(id, 1, true) then missing[#missing+1] = id end
end
need(#missing == 0, "needs the brief names but no body names: " .. table.concat(missing, ", "))
return report("the whole change: ubc, every statement, only stakeholder files, " .. (named == 0 and "and the brief names no need" or "every need the brief names"))
''', {"fdiff": "fdiff", "fubc": "fubc", "statements": "statements", "given_stakeholders": "given_stakeholders", "brief": "brief"})
model("describe", "final",
      "Write the pull request for this change: a title in the imperative, then what changed and why, what was checked (the final check and the loop's verdicts), and every open defect the notes list, said plainly. If the final check says FAIL or the notes list an open defect, say in the first sentence that the change is not ready.",
      {"brief": "brief", "diff": "fdiff", "final": "v_final", "verdict": "judge"}, role="describing")
person("approval", "final", "approve", {"description": "describe", "final": "v_final", "verdict": "judge"})

# ---- documents -------------------------------------------------------------------------------------
for name, (params, desc) in TOOLS.items():
    types[name] = dict(req=list(params), desc=desc)
types["approve"] = dict(req=["description", "final", "verdict"], desc="A person reads the proposed pull request and approves it or not. Nothing is published or merged: the run is a test.")

with open(os.path.join(OUT, "types.toml"), "w") as f:
    for t, d in types.items():
        f.write("[types.%s]\n" % t)
        f.write("required = { %s }\n" % ", ".join('%s = "note"' % p for p in d["req"]))
        f.write('output = "note"\n')
        if d.get("desc"):
            f.write("description = %s\n" % json.dumps(d["desc"]))
        if d.get("routes"):
            f.write("routes = true\n")
        if d.get("standing"):
            f.write("standing = true\n")
        f.write("\n")


def bound(p, src):
    if isinstance(src, tuple):
        return '%s = { from = "%s", input = "%s" }' % (p, src[0], src[1])
    return '%s = "%s"' % (p, src)


with open(os.path.join(OUT, "flow.toml"), "w") as f:
    f.write('name = "record-goals"\noutput = "approval"\n\n')
    for name, t, b, calls, kind, stage in instances:
        f.write("[instances.%s]\nnode_type = \"%s\"\n" % (name, t))
        b = {p: src for p, src in b.items() if src is not None}
        if b:
            f.write("bindings = { %s }\n" % ", ".join(bound(p, src) for p, src in b.items()))
        if calls:
            f.write("calls = [%s]\n" % ", ".join('"%s"' % c for c in calls))
        f.write("\n")
for t, lua in scripts.items():
    open(os.path.join(OUT, t + ".lua"), "w").write(lua)
M = '{ model = "openai::deepseek/deepseek-v4.1-flash", endpoint = "https://openrouter.ai/api/v1/", key_env = "OPEN_ROUTER_API_KEY" }'
roles.add("judging")
open(os.path.join(OUT, "models.toml"), "w").write("[roles]\n" + "".join("%s = %s\n" % (r, M) for r in sorted(roles)))
with open(os.path.join(OUT, "manifest.toml"), "w") as f:
    f.write('workflow = "flow.toml"\ntypes = ["types.toml"]\nbudget = 600\npersons = ["approve"]\n\n[scripts]\n')
    for t in scripts:
        f.write('%s = "%s.lua"\n' % (t, t))
    f.write('\n[tools]\nrun_command = "run"\nread_file = "read"\nwrite_file = "write"\nrun_ubc = { action = "run", image = "%s", container = "ubc" }\n\n[limits]\nmodel_calls = 15\n' % UBC_IMAGE)
open(os.path.join(OUT, "grants.template.toml"), "w").write(
    'image = "@GIT_IMAGE@"\nimages = ["@UBC_IMAGE@"]\nactions = ["read", "write", "run"]\nnetwork = true\n@TRUST@\n\n[folders.repo]\npath = "@WORK@"\nwritable = true\n\n[limits]\nseconds = 120\noutput = 32000\n')
os.makedirs(os.path.join(OUT, "arguments"), exist_ok=True)
for inst, param, text in arguments:
    open(os.path.join(OUT, "arguments", "%s.%s.txt" % (inst, param)), "w").write(text)
json.dump([dict(name=n, type=t, bindings=b, calls=c, kind=k, stage=s) for n, t, b, c, k, s in instances],
          open(os.path.join(OUT, "graph.json"), "w"), indent=1)
kinds = {}
for i in instances:
    kinds[i[4]] = kinds.get(i[4], 0) + 1
print(len(instances), "instances", kinds, len(arguments), "arguments")
