local given, host = ...

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
local base = table.concat({
  "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\nName the goal below as a stakeholder requirement of this repository. Answer with exactly five lines:\nID: STKH_ followed by upper-case words joined by underscores, not used by any goal below\nTITLE: a short sentence saying what the goal gives, in the style of the titles below, without a full stop\nPRECEDENT: the id of the existing goal below most like this one in whose goal it is\nSTAKEHOLDER: the goal's STAKEHOLDER GIVEN if it is not unset; otherwise user, agent or maintainer: the stakeholder of that precedent, unless the definition below plainly says otherwise\nWHY: one sentence saying why that stakeholder, naming who would miss this goal if it were not met. The goal below says which goal of how many this is and which pass. If its FEEDBACK is not 'none', a reviewer found what it says wrong with the last attempt at this goal: correct whatever of it concerns your answer.",
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## stakeholders\n\n" .. given.stakeholders:render(),
  "\n\n## goals\n\n" .. given.goals:render(),
  "\n\n## file_goals\n\n" .. given.file_goals:render()
})
local out, why = nil, nil
for attempt = 1, 3 do
  local text = base
  if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
  out = host.complete('drafting', host.text(host.output, text)):render()
  fails = {}

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

  if #fails == 0 then break end
  why = table.concat(fails, "\n")
end
return host.text(host.output, out)
