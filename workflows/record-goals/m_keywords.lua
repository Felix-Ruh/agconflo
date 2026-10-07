local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local base = table.concat({
  help.NO_TOOLS .. "Choose words to search the repository's decisions for any that bear on the goal below. Answer with 4 to 8 lowercase words, one per line, each a single word of letters only, nothing else. Choose words specific to what the goal asks for, which a decision about it would use." .. help.GOAL_NOTE,
  "\n\n## goal\n\n" .. given.goal:render()
})
return help.ask(host, 'drafting', base, function(out, need, clean)
  clean(out, "the answer")
  local n = 0
  for _, l in ipairs(lines(trim(out))) do
    n = n + 1
    need(trim(l):match("^[a-z]+$") ~= nil, "not one lowercase word: " .. trim(l))
  end
  need(n >= 4 and n <= 8, "expected 4 to 8 words, got " .. n)
end)
