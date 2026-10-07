local given, host = ...
local help = require('help')
local base = table.concat({
  help.NO_TOOLS .. "Choose the one file under repo/docs/stakeholder/ the goal below belongs in, from the listing of those files with their introductions and their goals. Answer with a first line 'FILE: ' followed by the path exactly as listed, then one sentence saying why, naming goals already in that file." .. help.GOAL_NOTE,
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## listing\n\n" .. given.listing:render()
})
return help.ask(host, 'drafting', base, function(out, need, clean)
  clean(out, "the answer")
  local f = out:match("FILE:%s*(%S+)")
  need(f ~= nil, "there is no line starting 'FILE: '")
  if f then need(given.listing:render():find("== " .. f .. ":", 1, true) ~= nil, f .. " is not a file in the listing") end
end)
