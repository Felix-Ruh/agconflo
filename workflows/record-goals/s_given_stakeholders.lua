local given, host = ...
local lines = require('help').lines
local out, seen = {}, {}
for _, l in ipairs(lines(given.brief:render())) do
  local s = l:match('"(Agconflo shall[^"]*)"')
  if s and not seen[s] then
    seen[s] = true
    out[#out+1] = l:match("%(stakeholder:%s*(%a+)%)") or "unset"
  end
end
return host.text(host.output, table.concat(out, "\n"))
