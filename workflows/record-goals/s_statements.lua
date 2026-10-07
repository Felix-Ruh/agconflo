local given, host = ...
local out, seen = {}, {}
for s in given.brief:render():gmatch('"(Agconflo shall[^"]*)"') do
  if not seen[s] then seen[s] = true; out[#out+1] = s end
end
return host.text(host.output, table.concat(out, "\n"))
