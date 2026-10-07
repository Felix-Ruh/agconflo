local given, host = ...
local help = require('help')
local trim = help.trim
local base = table.concat({
  help.NO_TOOLS .. "List every sentence of the draft's body below that states how Agconflo behaves, what it already does, or what another need says, leaving out sentences that name a need by its id (another step checks those), sentences giving the reason the goal exists and sentences saying what the goal deliberately does not say. For each, one line: 'C<n>: \"<the sentence>\" - SUPPORTED BY <ID>', naming a need listed below that says it, or 'C<n>: \"<the sentence>\" - UNSUPPORTED' when none does. A need supports a sentence only if it says the same thing, not merely the same topic. If there is no such sentence, answer exactly NONE.",
  "\n\n## draft\n\n" .. given.draft:render(),
  "\n\n## named\n\n" .. given.named:render(),
  "\n\n## goals\n\n" .. given.goals:render(),
  "\n\n## decs\n\n" .. given.decs:render(),
  "\n\n## current\n\n" .. given.current:render()
})
return help.ask(host, 'reviewing', base, function(out, need, clean)
  clean(out, "the answer")
  local known = given.goals:render() .. "\n" .. given.decs:render() .. "\n" .. given.named:render() .. "\n" .. given.current:render()
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
end)
