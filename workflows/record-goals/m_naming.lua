local given, host = ...
local help = require('help')
local base = table.concat({
  help.NO_TOOLS .. "Name the goal below as a stakeholder requirement of this repository. Answer with exactly five lines:\nID: STKH_ followed by upper-case words joined by underscores, not used by any goal below\nTITLE: a short sentence saying what the goal gives, in the style of the titles below, without a full stop\nPRECEDENT: the id of the existing goal below most like this one in whose goal it is\nSTAKEHOLDER: the goal's STAKEHOLDER GIVEN if it is not unset; otherwise user, agent or maintainer: the stakeholder of that precedent, unless the definition below plainly says otherwise\nWHY: one sentence saying why that stakeholder, naming who would miss this goal if it were not met." .. help.GOAL_NOTE,
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## stakeholders\n\n" .. given.stakeholders:render(),
  "\n\n## goals\n\n" .. given.goals:render(),
  "\n\n## file_goals\n\n" .. given.file_goals:render()
})
return help.ask(host, 'drafting', base, function(out, need, clean)
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
end)
