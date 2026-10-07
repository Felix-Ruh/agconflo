local given, host = ...
local help = require('help')
local rows = given.goals:render() .. "\n" .. given.decs:render() .. "\n" .. given.named:render()
local cur = given.current:render()
local function statement_of(id)
  local s = rows:match("\n" .. id .. "%s%s+.-%s%s+([^\n]-)%s*\n")
  if s then s = s:gsub("%s+%d+$", ""):gsub("%s%s+%a+$", "") end
  if not s then s = cur:match(":id: " .. id .. "\n.-:statement: ([^\n]*)") end
  return s
end
local out, k = {}, 0
for _, sent in ipairs(help.body_sentences(given.draft:render())) do
  for id in sent:gmatch("``([A-Z][A-Z0-9]*_[A-Z0-9_]+)``") do
    local st = statement_of(id)
    if st then k = k + 1; out[#out+1] = "P" .. k .. ": " .. sent .. "\n    " .. id .. " says: " .. st end
  end
end
if #out == 0 then return host.text(host.output, "none") end
return host.text(host.output, table.concat(out, "\n"))
