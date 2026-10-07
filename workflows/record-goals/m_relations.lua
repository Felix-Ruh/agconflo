local given, host = ...
local help = require('help')
local trim = help.trim
local base = table.concat({
  help.NO_TOOLS .. "Each pair below is a sentence from a new requirement's body and the statement of a need it names. For each pair, say whether the sentence describes that need and its relation to the new goal correctly, given only what the need's statement says: one line each, 'P<k>: CORRECT' or 'P<k>: WRONG - <what the statement says instead>'. If the pairs are 'none', answer exactly NONE.",
  "\n\n## pairs\n\n" .. given.pairs:render()
})
return help.ask(host, 'reviewing', base, function(out, need, clean)
  clean(out, "the answer")
  local p = given.pairs:render()
  if trim(p) == "none" then need(trim(out) == "NONE", "there are no pairs, so the answer is NONE")
  else
    for k in p:gmatch("P(%d+):") do
      need(out:find("P" .. k .. ": CORRECT", 1, true) or out:find("P" .. k .. ": WRONG", 1, true), "P" .. k .. " is not answered CORRECT or WRONG")
    end
  end
end)
