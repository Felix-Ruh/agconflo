local given, host = ...
local help = require('help')
local trim = help.trim
local base = table.concat({
  help.NO_TOOLS .. "Below is the task, which may carry a maintainer's note, and a draft for one of its goals. List each sentence of the maintainer's note that concerns this goal, and say whether the draft keeps it: one line each, 'N<k>: \"<the note's sentence, copied exactly>\" - KEPT', or '- MISSING' when the draft does not say it, or '- CONTRADICTED' when the draft says the opposite, then ' - ' and the draft's words that show it. If the task has no maintainer's note, or none of its sentences concerns this goal, answer exactly NONE.",
  "\n\n## brief\n\n" .. given.brief:render(),
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## draft\n\n" .. given.draft:render()
})
return help.ask(host, 'reviewing', base, function(out, need, clean)
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
end)
