local given, host = ...
local trim = require('help').trim
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
