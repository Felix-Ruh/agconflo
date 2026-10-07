local given, host = ...
local help = require('help')
local base = table.concat({
  help.NO_TOOLS .. "Say after which goal of the file the goal below goes: beside the goals it is most closely related to. Answer with exactly one line 'AFTER: ' followed by an id from the file's goals listed below, in their order, and nothing else." .. help.GOAL_NOTE,
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## file_goals\n\n" .. given.file_goals:render(),
  "\n\n## neighbours\n\n" .. given.neighbours:render()
})
return help.ask(host, 'drafting', base, function(out, need, clean)
  clean(out, "the answer")
  local a = out:match("AFTER:%s*(STKH_[A-Z0-9_]+)")
  need(a ~= nil, "no line 'AFTER: <id>'")
  if a then need(("\n" .. given.file_goals:render()):find("\n" .. a .. " ", 1, true) ~= nil, a .. " is not a goal in the file") end
end)
